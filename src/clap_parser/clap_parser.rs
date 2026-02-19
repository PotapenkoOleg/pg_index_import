
use clap::{Args, Parser, value_parser};

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

    // region Export
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
    // endregion

    // region Import
    #[arg(
        long,
        short = 'I',
        help = "Current directory sub directory for input files",
        default_value = "INPUT"
    )]
    pub input_dir: Option<String>,

    #[arg(
        long,
        short = 'r',
        default_value = "2",
        value_parser = value_parser!(u32).range(1..=10),
        help = "Number of threads from 1 to 10"
    )]
    pub threads: u32,

    #[arg(
        long,
        short = 'T',
        default_value = "24",
        value_parser = value_parser!(u64).range(1..=72),
        help = "Command timeout in hours from 1 to 72"
    )]
    pub timeout_in_hours: u64,
    // endregion
}

#[derive(Args, Debug)]
#[group(multiple = false)]
pub struct ExclusiveOptions {
    #[arg(long, short, help = "Export indexes from SQL Server to files")]
    pub export: Option<bool>,

    #[arg(
        long,
        short,
        help = "Import indexes from files to Postgres",
        default_value = "true"
    )]
    pub import: Option<bool>,
}
