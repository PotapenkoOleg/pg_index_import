use crate::clap_parser::clap_parser::Cli;
use crate::config_provider::{Config, ConfigProvider};
use crate::helpers::{print_banner, print_separator};
use crate::postgres_provider::postgres_provider::PostgresProvider;
use crate::settings::settings::Settings;
use crate::shared::file_utils::{
    ensure_directory_exists_and_empty, list_files, read_file, write_index_to_file,
};
use crate::sql_server_provider::sql_server_provider::SqlServerProvider;
use clap::Parser;
use colored::Colorize;
use futures_util::future::join_all;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, process};
use tokio::task::JoinHandle;
use tokio::time::Instant;

mod clap_parser;
mod config_provider;
mod helpers;
mod postgres_provider;
mod settings;
mod shared;
mod sql_server_provider;
mod version;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let cli = Cli::parse();
    print_separator();
    print_banner();
    print_separator();
    // region Command Line Args
    let settings = Arc::new(Settings::from_args(&cli));
    println!("{}", settings);
    // endregion
    print_separator();
    // region Config File
    println!("Loading Config File: <{}> ", &cli.config_file);
    let config_provider = ConfigProvider::new(&cli.config_file);
    let file_load_result = config_provider.read_config().await;
    if file_load_result.is_err() {
        eprintln!("{}", file_load_result.err().unwrap().to_string().red());
        process::exit(1);
    }
    let config = file_load_result.ok().unwrap();
    println!("{}", "DONE Loading Config File".green());
    // endregion
    print_separator();
    // region Processing
    match (cli.exclusive_options.export, cli.exclusive_options.import) {
        (Some(_), None) => {
            export_indexes(settings, config).await;
        }
        (None, Some(_)) => {
            import_indexes(settings, config).await;
        }
        _ => unreachable!(),
    }
    // endregion
    print_separator();
}

async fn export_indexes(settings: Arc<Settings>, config: Config) {
    println!("Creating Sql Server Provider ...");
    let source_db_provider = SqlServerProvider::new(&config.get_source_database_as_ref());
    println!("{}", "DONE Creating Sql Server Provider".green());
    let schema_vec = if settings.get_source_schema_name_as_ref().eq("*") {
        let result = source_db_provider.get_all_schemas().await.unwrap();
        result
    } else {
        vec![settings.get_source_schema_name_as_ref().to_string()]
    };
    let mut current_dir: PathBuf = env::current_dir().unwrap();
    current_dir.push(settings.get_output_dir_as_ref().clone().unwrap());
    println!(
        "Output directory: <{}>",
        current_dir.to_str().unwrap().yellow()
    );
    for schema in schema_vec {
        print_separator();
        println!("Source Schema: <{}>", schema.yellow());
        let table_vec = if settings.get_source_table_name_as_ref().eq("*") {
            source_db_provider
                .get_all_tables_in_schema(&schema)
                .await
                .unwrap()
        } else {
            vec![settings.get_source_table_name_as_ref().to_string()]
        };
        let mut current_dir = current_dir.clone();
        current_dir.push(&schema);
        ensure_directory_exists_and_empty(&current_dir)
            .await
            .unwrap();
        for table in table_vec {
            print_separator();
            println!("Source Table: <{}>", table.yellow());
            print_separator();
            let mut current_dir = current_dir.clone();
            current_dir.push(&table);
            ensure_directory_exists_and_empty(&current_dir)
                .await
                .unwrap();
            let indexes = source_db_provider
                .get_all_indexes_in_table(&schema, &table)
                .await
                .unwrap();
            for (index_name, index_def) in indexes {
                let file_name = &current_dir
                    .join(index_name.replace("[", "").replace("]", "").to_string() + ".sql");
                println!(
                    "Exporting Index: <{}> to <{}>",
                    index_name.yellow(),
                    file_name.to_str().unwrap().yellow()
                );
                write_index_to_file(file_name, &index_def).await.unwrap();
            }
        }
    }
}

async fn import_indexes(settings: Arc<Settings>, config: Config) {
    // region Get a file list
    println!("Getting a list of files in input directory ...");
    let input_dir_name = settings.get_input_dir_as_ref().clone().unwrap();
    let input_dir = PathBuf::from(input_dir_name);
    let mut files_list: Vec<PathBuf> = Vec::new();
    list_files(&input_dir, &mut files_list)
        .await
        .unwrap_or_else(|e| {
            eprintln!("{}", e.to_string().red());
            process::exit(1);
        });
    files_list = files_list
        .into_iter()
        .filter(|f| f.extension().unwrap() == "sql")
        .collect();
    println!(
        "{}",
        "DONE Getting a list of files in input directory".green()
    );
    // endregion
    print_separator();
    // region Postgres Connection Pool
    println!("Creating Postgres Connection Pool ...");
    let postgres_provider = PostgresProvider::new(&config.get_target_database_as_ref());
    let postgres_pool_result = postgres_provider
        .create_connection_pool(settings.get_threads(), settings.get_timeout())
        .await;
    if postgres_pool_result.is_err() {
        eprintln!("{}", postgres_pool_result.err().unwrap().to_string().red());
        process::exit(1);
    }
    let postgres_pool = postgres_pool_result.ok().unwrap();
    println!("{}", "DONE Creating Postgres Connection Pool".green());
    // endregion
    print_separator();
    // region Indexes Import
    println!("Importing Indexes ...");
    let mut handles = Vec::new();
    let (tx, rx) = flume::unbounded();
    for _ in 0..settings.get_threads() {
        let rx = rx.clone();
        let postgres_pool = postgres_pool.clone();
        let handle: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
            while let Ok((file_name, index_query)) = rx.recv_async().await {
                print_separator();
                let now = Instant::now();
                println!("Importing Index: <{}>", file_name);
                println!("{}", index_query);

                let postgres_connection = postgres_pool.get().await?;
                // let postgres_client = postgres_connection.client();
                // postgres_client.execute("SET statement_timeout TO 10000", &[]).await?;
                // postgres_client.execute("SET lock_timeout TO 10000", &[]).await?;
                // postgres_client.execute("SET idle_in_transaction_session_timeout TO 10000", &[]).await?;

                match postgres_connection.execute(&index_query, &[]).await {
                    Ok(_) => {
                        println!("{}", "Index imported successfully".green());
                    }
                    Err(e) => {
                        eprintln!("{}: {}", "Error importing index".red(), e.to_string().red());
                    }
                }

                let elapsed = now.elapsed();
                println!("Elapsed: {:.2?}", elapsed);
            }
            Ok(())
        });
        handles.push(handle);
    }
    for file in files_list {
        let file_content = read_file(&file).await.unwrap();
        let file = file.to_str().unwrap().to_string();
        tx.send_async((file, file_content)).await.unwrap();
    }
    drop(tx); // finish sending data

    let thread_results = join_all(handles).await;
    for thread_result in thread_results {
        if thread_result.is_err() {
            eprintln!(
                "Error in thread: {}",
                thread_result.err().unwrap().to_string().red()
            );
        }
    }

    print_separator();
    println!("{}", "DONE Importing Indexes".green());
    //endregion
}
