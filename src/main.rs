use crate::clap_parser::clap_parser::Cli;
use crate::config_provider::ConfigProvider;
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
    let args = Cli::parse();

    print_separator();
    print_banner();
    print_separator();
    // region Command Line Args
    let mut settings = Arc::new(Settings::from_args(&args));
    println!("{}", settings);
    // endregion
    print_separator();
    // region Config File
    println!("Loading Config File: <{}> ", &args.config_file);
    let config_provider = ConfigProvider::new(&args.config_file);
    let file_load_result = config_provider.read_config().await;
    if file_load_result.is_err() {
        eprintln!("{}", file_load_result.err().unwrap().to_string().red());
        process::exit(1);
    }
    let config = file_load_result.ok().unwrap();
    println!("{}", "DONE Loading Config File".green());
    // endregion
    print_separator();

    // region Export Indexes
    // TODO:
    // match (cli.exclusive_options.option_a, cli.exclusive_options.option_b) {
    //     (Some(a), None) => println!("Got A: {}", a),
    //     (None, Some(b)) => println!("Got B: {}", b),
    //     _ => unreachable!(), // clap enforces that only one can be present
    // }
    if args.exclusive_options.export.unwrap() {
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
        current_dir.push(&args.output_dir.unwrap());
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
                // region Table name setup
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
                // endregion
            }
        }
    }
    // endregion
    print_separator();
}
