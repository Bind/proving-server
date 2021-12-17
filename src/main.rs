use rocket::http::Status;
mod prover;
mod storage;
mod utils;
use rocket::serde::json::Json;
use std::io::copy;
use std::{fs::File, path::PathBuf};
use storage::{EnvConfig, ProverConfig};
extern crate dotenv;

use dotenv::dotenv;
use std::env;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}
pub fn get_zkey_path(prover: ProverConfig, config: EnvConfig) -> PathBuf {
    let mut path = get_path_from_prover(prover, config).unwrap();
    path.set_extension("zkey");
    return path;
}
pub fn get_wasm_path(prover: ProverConfig, config: EnvConfig) -> PathBuf {
    let mut path = get_path_from_prover(prover, config).unwrap();
    path.set_extension("wasm");
    return path;
}

pub fn get_path_from_prover(
    prover: ProverConfig,
    config: EnvConfig,
) -> Result<PathBuf, std::io::Error> {
    let mut path = PathBuf::from(config.zk_file_path.clone());
    path = path.join(prover.version.clone());
    utils::files::create_dir(&path);
    path = path.join(prover.name.clone());
    Ok(path)
}

pub async fn fetch_file(path: PathBuf, url: String) -> Status {
    let resp = match reqwest::get(url).await {
        Ok(res) => res,
        Err(_) => return Status::BadRequest,
    };
    let mut dest = File::create(path).unwrap();
    let content = resp.bytes().await.unwrap();

    copy(&mut content.as_ref(), &mut dest).unwrap();
    return Status::Accepted;
}

#[post("/prover", format = "json", data = "<prover>")]
pub async fn add_prover_handler(
    db: &rocket::State<storage::Db>,
    config: &rocket::State<storage::Config>,
    prover: Json<storage::ProverConfig>,
) -> Status {
    let mut db = db.lock().await;
    let config = config.lock().await;
    let prover = prover.into_inner();

    let wasm_path = get_wasm_path(prover.clone(), config.clone());
    let zkey_path = get_zkey_path(prover.clone(), config.clone());
    db.insert(prover.name.clone(), prover.clone());

    fetch_file(wasm_path, prover.path_to_wasm.clone()).await;
    fetch_file(zkey_path, prover.path_to_zkey.clone()).await;
    return Status::Accepted;
}
#[get("/prover")]
pub async fn list_provers_handler(
    db: &rocket::State<storage::Db>,
) -> Option<Json<Vec<storage::ProverConfig>>> {
    let prover_hm = db.lock().await;
    let provers: Vec<storage::ProverConfig> = prover_hm.values().cloned().collect();
    println!("database {:?}", provers);
    return Some(Json(provers));
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    env::vars();
    rocket::build()
        .manage(storage::init_storage())
        .manage(storage::init_config())
        .mount("/", routes![index])
        .mount("/v1/", routes![add_prover_handler, list_provers_handler])
}
