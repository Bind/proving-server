use rocket::fairing::{Fairing, Info, Kind};

use rocket::serde::{json::Json, Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ProverConfig {
    pub name: String,
    pub version: String,
    pub path_to_wasm: String,
    pub path_to_zkey: String,
    pub builder_params: Vec<String>,
}

pub type Db = Arc<Mutex<HashMap<String, ProverConfig>>>;

pub fn init_storage() -> Db {
    return Arc::new(Mutex::new(HashMap::new()));
}
