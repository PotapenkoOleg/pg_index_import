use async_trait::async_trait;

#[async_trait]
pub trait DbProvider: Send + Sync {
    // async fn get_all_schemas(&self) -> anyhow::Result<Vec<String>>;
    //
    // async fn get_all_tables_in_schema(&self, schema_name: &str) -> anyhow::Result<Vec<String>>;
}
