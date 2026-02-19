use crate::clap_parser::clap_parser::Cli;
use std::fmt;

pub struct Settings {
    source_schema_name: String,
    source_table_name: String,
    output_dir: Option<String>,
    input_dir: Option<String>,
    threads: u32,
    timeout_in_hours: u64,
}

impl Settings {
    pub fn from_args(cli: &Cli) -> Self {
        let source_schema_name = cli.source_schema.clone();

        let mut source_table_name = cli.source_table.clone();
        if source_table_name.eq("*") {
            source_table_name = "*".to_string();
        }

        let output_dir = cli.output_dir.clone();

        let input_dir = cli.input_dir.clone();
        let threads = cli.threads;
        let timeout_in_hours = cli.timeout_in_hours;

        Settings {
            source_schema_name,
            source_table_name,
            output_dir,
            input_dir,
            threads,
            timeout_in_hours,
        }
    }

    // region Getters
    pub fn get_source_schema_name_as_ref(&self) -> &String {
        &self.source_schema_name
    }

    pub fn get_source_table_name_as_ref(&self) -> &String {
        &self.source_table_name
    }

    pub fn get_output_dir_as_ref(&self) -> &Option<String> {
        &self.output_dir
    }

    pub fn get_input_dir_as_ref(&self) -> &Option<String> {
        &self.input_dir
    }

    pub fn get_threads(&self) -> u32 {
        self.threads
    }

    pub fn get_timeout(&self) -> u64 {
        self.timeout_in_hours
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Source schema name: <{}>", self.source_schema_name)?;
        writeln!(f, "Source table name: <{}>", self.source_table_name)?;
        writeln!(
            f,
            "Output directory: <{}>",
            self.output_dir.as_ref().unwrap()
        )?;
        writeln!(f, "Input directory: <{}>", self.input_dir.as_ref().unwrap())?;
        writeln!(f, "Threads: <{}>", self.threads)?;
        writeln!(f, "Timeout: <{}>", self.timeout_in_hours)?;
        Ok(())
    }
}
