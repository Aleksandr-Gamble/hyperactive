use std::{env, vec::Vec};
//use async_recursion::async_recursion;
//use chrono;
pub use tokio_postgres::{Config, NoTls};
pub use mobc::Pool;
pub use mobc_postgres::PgConnectionManager;
use crate::server::{ErrHTTP, GenericError};

pub struct DBConfig {
    // This struct describes how to connect to postgres
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

impl DBConfig {
    pub fn new_from_env() -> Self {
        let host = match env::var("PSQL_HOST") {
            Ok(var) => var,
            Err(_) => "127.0.0.1".to_string(),
        };
        let port = match env::var("PSQL_PORT") {
            Ok(var) => var,
            Err(_) => "5432".to_string(),
        };
        let password = match env::var("PSQL_PW") {
            Ok(var) => var,
            Err(_) => "".to_string(),
        };
        let user = match env::var("PSQL_USER") {
            Ok(var) => var,
            Err(_) => "postgres".to_string(),
        };
        let database = match env::var("PSQL_DB") {
            Ok(var) => var,
            Err(_) => "postgres".to_string(),
        };
        DBConfig {
            host: host,
            port: port.parse::<u16>().unwrap(),
            user: user,
            password: password,
            database: database,
        }
    }
}


/// This struct represents pool database connections
#[derive(Clone)]
pub struct NoTlsPool {
    pub pool: Pool<PgConnectionManager<NoTls>>
}

impl NoTlsPool { // instantiation methods
    pub async fn from_config(db_config: &DBConfig) -> Result<Self, GenericError> {
        //let conn_string = format!("postgres://{}:{}@127.0.0.1:{}/loxodonta",&config.user, &config.password, &config.port );
        let mut config = Config::new();
        config.user(&db_config.user);
        config.password(&db_config.password);
        config.dbname(&db_config.database);
        config.host(&db_config.host);
        config.port(db_config.port);

        let manager = PgConnectionManager::new(config, NoTls);
        let pool = Pool::builder().max_open(20).max_idle(5).build(manager);
        let _client = pool.get().await?; // Try to get a connection to ensure you can connect
        Ok(NoTlsPool{pool: pool})
    }

    pub async fn new_from_env() -> Result<Self, GenericError> {
        // de-facto way of creating a new instance when you have environment variables
        let db_config = DBConfig::new_from_env();
        NoTlsPool::from_config(&db_config).await
    }
}


pub fn ts_expression(phrase: &str) -> String {
    // Given a phrase like "crimson thread", convert it to a TS expression
    let mut prefixes = Vec::new();
    for word in phrase.to_lowercase().split_whitespace() {
        let mut prefix = word.to_string();
        prefix.push_str(":*");
        prefixes.push(prefix);
    }
    let ts_expression = prefixes.join(" & ");
    ts_expression
}




