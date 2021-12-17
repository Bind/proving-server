use crate::prover::CircuitProver;
use rocket::serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ProverConfig {
    pub name: String,
    pub version: String,
    pub path_to_wasm: String,
    pub path_to_zkey: String,
    pub path_to_r1cs: String,
    pub builder_params: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct EnvConfig {
    pub zk_file_path: String,
}

pub type Db = Arc<Mutex<HashMap<String, ProverConfig>>>;
pub type Provers = Arc<Mutex<HashMap<String, CircuitProver>>>;

pub type Config = Arc<Mutex<EnvConfig>>;
pub fn init_storage() -> Db {
    return Arc::new(Mutex::new(HashMap::new()));
}
pub fn init_provers() -> Provers {
    return Arc::new(Mutex::new(HashMap::new()));
}
pub fn init_config() -> Config {
    let zk_file_path = env::var("ZK_FILE_PATH").unwrap();
    let conf = EnvConfig {
        zk_file_path: zk_file_path,
    };
    return Arc::new(Mutex::new(conf));
}
