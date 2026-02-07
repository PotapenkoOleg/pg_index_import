use crate::clap_parser::clap_parser::Cli;
use std::fmt;

pub struct Settings {
    source_schema_name: String,
    source_table_name: String,
}

impl Settings {
    pub fn from_args(args: &Cli) -> Self {
        let source_schema_name = args.source_schema.clone();

        let mut source_table_name = args.source_table.clone();
        if source_table_name.eq("*") {
            source_table_name = "*".to_string();
        }

        Settings {
            source_schema_name,
            source_table_name,
        }
    }

    // region Getters
    pub fn get_source_schema_name_as_ref(&self) -> &String {
        &self.source_schema_name
    }

    pub fn get_source_table_name_as_ref(&self) -> &String {
        &self.source_table_name
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Source schema name: <{}>", self.source_schema_name)?;
        writeln!(f, "Source table name: <{}>", self.source_table_name)?;
        Ok(())
    }
}
