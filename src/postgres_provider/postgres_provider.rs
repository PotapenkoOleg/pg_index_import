use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{Config, NoTls};

use crate::config_provider::TargetDatabase;

const MAX_CONNECTION_LIFETIME_IN_HOURS: u64 = 24 * 2; // 2 days
const IDLE_TIMEOUT_IN_HOURS: u64 = 24;

pub struct PostgresProvider {
    config: Config,
}

impl PostgresProvider {
    pub fn new(source_database: &TargetDatabase) -> Self {
        let host = source_database.get_host_as_ref();
        let port = source_database.get_port_as_ref().clone();
        let dbname = source_database.get_database_as_ref();
        let user = source_database.get_user_as_ref();
        let password = source_database.get_password_as_ref();
        let mut config = Config::new();
        config.host(host);
        config.port(port);
        config.dbname(dbname);
        config.user(user);
        config.password(password);
        config.keepalives(true);
        PostgresProvider {
            config,
        }
    }

    pub async fn create_connection_pool(
        &self,
        threads: u32,
        timeout_in_hours: u64,
    ) -> anyhow::Result<Pool<PostgresConnectionManager<NoTls>>> {
        let manager = PostgresConnectionManager::new(self.config.clone(), NoTls);
        let pool = Pool::builder()
            .max_size(threads)
            .max_lifetime(std::time::Duration::from_hours(
                MAX_CONNECTION_LIFETIME_IN_HOURS,
            ))
            .idle_timeout(std::time::Duration::from_hours(IDLE_TIMEOUT_IN_HOURS))
            .connection_timeout(std::time::Duration::from_hours(timeout_in_hours))
            .build(manager)
            .await?;
        Ok(pool)
    }
}

