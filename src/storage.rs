use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProverConfig {
    path_to_wasm: String,
    path_to_zkey: String,
    builder_params: Vec<String>,
}

pub type Db = Arc<Mutex<HashMap<String, ProverConfig>>>;

pub fn init_storage() -> Db {
    return Arc::new(Mutex::new(HashMap::new()));
}
