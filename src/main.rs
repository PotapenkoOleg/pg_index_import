use crate::clap_parser::clap_parser::Cli;
use crate::config_provider::{Config, ConfigProvider};
use crate::helpers::{print_banner, print_separator};
use crate::settings::settings::Settings;
use crate::shared::file_utils::{ensure_directory_exists_and_empty, write_index_to_file};
use crate::sql_server_provider::sql_server_provider::SqlServerProvider;
use clap::Parser;
use colored::Colorize;
use std::path::PathBuf;
use std::sync::Arc;
use std::{env, process};

mod clap_parser;
mod config_provider;
mod helpers;
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
            import_indexes().await;
        }
        _ => unreachable!(),
    }
    // endregion
    print_separator();
}

async fn export_indexes(settings: Arc<Settings>, config: Config) {
    println!("Getting Source DB configuration ...");
    let source_db_provider = SqlServerProvider::new(&config.get_source_database_as_ref());
    println!("{}", "DONE Source DB configuration".green());
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

async fn import_indexes() {}
