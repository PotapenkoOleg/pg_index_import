use clap::{Args, Parser, Subcommand};
use std::process::Output;

#[derive(Parser, Debug)]
#[command(
    author = "Oleg Potapenko",
    version,
    about = "Utility for extracting indexes from SQL Server and importing them to Postgres"
)]
pub struct Cli {
    #[command(flatten)]
    pub exclusive_options: ExclusiveOptions,

    #[arg(
        long,
        short = 'f',
        default_value = "pg_index_import.toml",
        help = "Configuration file name"
    )]
    pub config_file: String,

    #[arg(
        long,
        short = 's',
        help = "Source schema name. Use '*' to import all schemas",
        default_value = "*"
    )]
    pub source_schema: String,

    #[arg(
        long,
        short = 't',
        help = "Source table name. Use '*' to import all tables in schema",
        default_value = "*"
    )]
    pub source_table: String,

    #[arg(
        long,
        short,
        help = "Current directory sub directory for output files",
        default_value = "OUTPUT"
    )]
    pub output_dir: Option<String>,
}

#[derive(Args, Debug)]
#[group(multiple = false)]
pub struct ExclusiveOptions {
    #[arg(long, short, help = "Export indexes from SQL Server to files", default_value = "true")]
    pub export: Option<bool>,

    #[arg(long, short, help = "Import indexes from files to Postgres")]
    pub import: Option<bool>,
}
