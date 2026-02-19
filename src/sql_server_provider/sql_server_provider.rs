use crate::config_provider::SourceDatabase;
use crate::sql_server_provider::sql_server_index_extract_query::SQL_SERVER_INDEX_EXTRACT_QUERY;
use crate::version::PRODUCT_NAME;
use anyhow::Result;
use futures_util::TryStreamExt;
use tiberius::{AuthMethod, Client, Config, EncryptionLevel, QueryItem};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;

pub struct SqlServerProvider {
    config: Config,
}

impl SqlServerProvider {
    pub fn new(source_database: &SourceDatabase) -> Self {
        let mut config = Config::new();
        config.host(source_database.get_host_as_ref());
        config.port(source_database.get_port_as_ref().clone());
        config.database(source_database.get_database_as_ref());
        config.authentication(AuthMethod::sql_server(
            source_database.get_user_as_ref(),
            source_database.get_password_as_ref(),
        ));
        config.trust_cert();
        config.readonly(true);
        config.application_name(PRODUCT_NAME);
        config.encryption(EncryptionLevel::NotSupported); // TODO: remove on PROD
        SqlServerProvider { config }
    }

    pub async fn get_all_schemas(&self) -> Result<Vec<String>> {
        let get_schemas_query = "SELECT SCHEMA_NAME, '' FROM INFORMATION_SCHEMA.SCHEMATA AS S WHERE S.SCHEMA_NAME NOT IN ('db_accessadmin','db_backupoperator','db_datareader','db_datawriter','db_ddladmin','db_denydatareader','db_denydatawriter','db_owner','db_securityadmin','guest','INFORMATION_SCHEMA','sys') ORDER BY S.SCHEMA_NAME;";
        self.execute_query(get_schemas_query).await
    }

    pub async fn get_all_tables_in_schema(&self, schema_name: &str) -> Result<Vec<String>> {
        let get_tables_query = format!(
            "SELECT TABLE_NAME, '' FROM INFORMATION_SCHEMA.TABLES AS T WHERE T.TABLE_SCHEMA = '{}' ORDER BY TABLE_NAME;",
            schema_name
        );
        self.execute_query(&get_tables_query).await
    }

    pub async fn get_all_indexes_in_table(
        &self,
        schema_name: &str,
        table_name: &str,
    ) -> Result<Vec<(String, String)>> {
        let schema = format!("DECLARE @SchemaName sysname = N'{}';", schema_name);
        let table = format!("DECLARE @TableName sysname = N'{}';", table_name);
        let get_indexes_query =
            format!("{}\n{}\n{}", schema, table, SQL_SERVER_INDEX_EXTRACT_QUERY);
        self.execute_query_2(&get_indexes_query).await
    }

    async fn execute_query(&self, query: &str) -> Result<Vec<String>> {
        let tcp = TcpStream::connect(&self.config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        let mut client = Client::connect(self.config.clone(), tcp.compat()).await?;
        let mut stream = client.query(query, &[]).await?;
        let mut result = Vec::new();
        while let Some(item) = stream.try_next().await? {
            match item {
                QueryItem::Row(row) => {
                    let data0: &str = row.get(0).unwrap();

                    result.push(data0.to_string());
                }
                _ => {}
            }
        }
        Ok(result)
    }

    async fn execute_query_2(&self, query: &str) -> Result<Vec<(String, String)>> {
        let tcp = TcpStream::connect(&self.config.get_addr()).await?;
        tcp.set_nodelay(true)?;
        let mut client = Client::connect(self.config.clone(), tcp.compat()).await?;
        let mut stream = client.query(query, &[]).await?;
        let mut result = Vec::new();
        while let Some(item) = stream.try_next().await? {
            match item {
                QueryItem::Row(row) => {
                    let data0: &str = row.get(0).unwrap();
                    let data1: &str = row.get(1).unwrap();
                    result.push((data0.to_string(), data1.to_string()));
                }
                _ => {}
            }
        }
        Ok(result)
    }
}