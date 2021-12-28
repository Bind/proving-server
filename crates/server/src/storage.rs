use crate::errors::ProvingServerError;
use crate::types::{CircuitProver, DatabaseMode, ProofInputs};
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

impl ProverConfig {
    pub fn validate_inputs(&self, inputs: &ProofInputs) -> Result<bool, ProvingServerError> {
        for param in &self.builder_params {
            if !inputs.contains_key(&param.clone()) {
                return Err(ProvingServerError::BadProofInputsError {
                    message: String::from(format!("{}", param.clone())),
                });
            }
        }
        return Ok(true);
    }
}

#[derive(Clone, Debug)]
pub struct EnvConfig {
    pub zk_file_path: String,
    pub db_config: DatabaseMode,
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
