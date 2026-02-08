use crate::clap_parser::clap_parser::Cli;
use std::fmt;

pub struct Settings {
    source_schema_name: String,
    source_table_name: String,
    output_dir: Option<String>,
}

impl Settings {
    pub fn from_args(cli: &Cli) -> Self {
        let source_schema_name = cli.source_schema.clone();

        let mut source_table_name = cli.source_table.clone();
        if source_table_name.eq("*") {
            source_table_name = "*".to_string();
        }

        let output_dir = cli.output_dir.clone();

        Settings {
            source_schema_name,
            source_table_name,
            output_dir,
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
        Ok(())
    }
}
