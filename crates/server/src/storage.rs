use crate::types::proof::Provers;
use crate::types::DatabaseMode;
use crate::types::{Config, Db, EnvConfig};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn init_storage() -> Db {
    return Arc::new(Mutex::new(HashMap::new()));
}
pub fn init_provers() -> Provers {
    return Arc::new(Mutex::new(HashMap::new()));
}
pub fn init_config() -> EnvConfig {
    let zk_file_path = env::var("ZK_FILE_PATH").unwrap();
    let db_config = match env::var("DB_FILE_PATH") {
        Ok(string) => DatabaseMode::File {
            path_to_file: string,
        },
        Err(_) => DatabaseMode::Memory,
    };
    let conf = EnvConfig {
        zk_file_path: zk_file_path,
        db_config: db_config,
    };
    return conf;
}
pub fn init_async_config() -> Config {
    let conf = init_config();
    return Arc::new(Mutex::new(conf));
}
