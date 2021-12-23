use rocket::http::Status;
mod db;
mod errors;
mod prover;
mod storage;
mod types;
mod utils;
use crate::errors::ProvingServerError;
use ark_circom::ethereum::Proof;
use rocket::serde::json::Json;
use std::io::copy;
use std::{fs::File, path::PathBuf};
use storage::{EnvConfig, ProverConfig};
use types::{to_eth_type, Abc, ProofInputs};
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
pub fn get_r1cs_path(prover: ProverConfig, config: EnvConfig) -> PathBuf {
    let mut path = get_path_from_prover(prover, config).unwrap();
    path.set_extension("r1cs");
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
    prover_storage: &rocket::State<storage::Provers>,
    config: &rocket::State<storage::Config>,
    prover: Json<storage::ProverConfig>,
) -> Status {
    let mut db = db.lock().await;
    let config = config.lock().await;
    let prover = prover.into_inner();

    let wasm_path = get_wasm_path(prover.clone(), config.clone());
    let zkey_path = get_zkey_path(prover.clone(), config.clone());
    let r1cs_path = get_r1cs_path(prover.clone(), config.clone());
    db.insert(prover.name.clone(), prover.clone());

    println!(
        "wasm:{:?} \nzkey:{:?} \nr1cs:{:?}",
        wasm_path.clone(),
        zkey_path.clone(),
        r1cs_path.clone()
    );
    fetch_file(wasm_path.clone(), prover.path_to_wasm.clone()).await;
    fetch_file(zkey_path.clone(), prover.path_to_zkey.clone()).await;
    fetch_file(r1cs_path.clone(), prover.path_to_r1cs.clone()).await;
    let p = types::CircuitProver::new_path(zkey_path, wasm_path, r1cs_path).unwrap();

    let mut prover_storage = prover_storage.lock().await;
    prover_storage.insert(prover.name.clone(), p);
    return Status::Ok;
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

#[post("/prove/<prover_name>", data = "<inputs>")]
pub async fn execute_prover(
    prover_storage: &rocket::State<storage::Provers>,
    prover_cfg: &rocket::State<storage::Db>,
    prover_name: &str,
    inputs: Json<ProofInputs>,
) -> Result<Json<Abc>, ProvingServerError> {
    println!("fetching prover");
    let prover_storage = prover_storage.lock().await;
    let p = prover_storage.get(prover_name).unwrap();
    println!("fetching prover config");
    let prover_cfg = prover_cfg.lock().await;
    let cfg = prover_cfg.get(prover_name).unwrap();

    let proof_inputs = inputs.into_inner();
    match cfg.validate_inputs(&proof_inputs) {
        Err(error) => return Err(error),
        _ => (),
    }

    println!("generating circuit");
    let circuit = prover::build_inputs(p.clone(), cfg.clone(), proof_inputs);
    let (proof, _) = prover::prove(circuit, &p.params).unwrap();
    // Check that the proof is valid

    return Ok(Json(to_eth_type(Proof::from(proof))));
}

#[launch]
fn rocket() -> _ {
    dotenv().ok();
    env::vars();
    println!("{:?}", env::vars());
    rocket::build()
        .manage(storage::init_storage())
        .manage(storage::init_config())
        .manage(storage::init_provers())
        .mount("/", routes![index])
        .mount(
            "/v1/",
            routes![add_prover_handler, list_provers_handler, execute_prover],
        )
}

#[cfg(test)]
mod test {
    use super::rocket;
    use super::storage::ProverConfig;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use std::collections::HashMap;
    fn max_distance(x1: i64, y1: i64, x2: i64, y2: i64) -> u64 {
        ((x1 - x2).pow(2) as f64 + (y1 - y2).pow(2) as f64).sqrt() as u64 + 1
    }
    #[test]
    fn test_add_prover_route() {
        let rocket_instance = rocket();
        let client = Client::tracked(rocket_instance).expect("valid rocket instance");

        let prover = ProverConfig {
            name: String::from("test"),
            version: String::from("0.0.1"),
            path_to_r1cs: String::from("https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.r1cs"),
            path_to_wasm: String::from("https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.wasm"),
            path_to_zkey: String::from("https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.zkey"),
            builder_params: vec![
                String::from("x1"),
                String::from("y1"),
                String::from("x2"),
                String::from("y2"),
                String::from("r"),
                String::from("distMax"),
                String::from("PLANETHASH_KEY"),
                String::from("SPACETYPE_KEY"),
                String::from("SCALE"),
                String::from("xMirror"),
                String::from("yMirror"),
            ],
        };
        let response = client.post("/v1/prover").json(&prover).dispatch();
        println!("{:?}", response.body());
        assert_eq!(response.status(), Status::Ok);
    }
    #[test]
    fn test_proof_generation() {
        let rocket_instance = rocket();
        let client = Client::tracked(rocket_instance).expect("valid rocket instance");
        let prover = ProverConfig {
            name: String::from("test"),
            version: String::from("0.0.1"),
            path_to_r1cs: String::from("https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.r1cs"),
            path_to_wasm: String::from("https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.wasm"),
            path_to_zkey: String::from("https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.zkey"),
            builder_params: vec![
                String::from("x1"),
                String::from("y1"),
                String::from("x2"),
                String::from("y2"),
                String::from("r"),
                String::from("distMax"),
                String::from("PLANETHASH_KEY"),
                String::from("SPACETYPE_KEY"),
                String::from("SCALE"),
                String::from("xMirror"),
                String::from("yMirror"),
            ],
        };
        let response = client.post("/v1/prover").json(&prover).dispatch();
        println!("{:?}", response.body());
        assert_eq!(response.status(), Status::Ok);

        let mut proof_request: HashMap<String, u64> = HashMap::new();
        proof_request.insert(String::from("x1"), 100);
        proof_request.insert(String::from("y1"), 100);
        proof_request.insert(String::from("x2"), 120);
        proof_request.insert(String::from("y2"), 120);
        proof_request.insert(String::from("r"), 8000);
        proof_request.insert(String::from("distMax"), max_distance(100, 100, 120, 120));
        proof_request.insert(String::from("PLANETHASH_KEY"), 1729);
        proof_request.insert(String::from("SPACETYPE_KEY"), 1730);
        proof_request.insert(String::from("xMirror"), false as u64);
        proof_request.insert(String::from("SCALE"), 16384);
        proof_request.insert(String::from("yMirror"), false as u64);
        let response = client
            .post("/v1/prove/test")
            .json(&proof_request)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        let response = client.post("/v1/prover").json(&prover).dispatch();
        println!("{:?}", response.body());
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    #[should_panic]
    fn test_bad_proof_generation() {
        let rocket_instance = rocket();
        let client = Client::tracked(rocket_instance).expect("valid rocket instance");

        let prover = ProverConfig {
            name: String::from("test"),
            version: String::from("0.0.1"),
            path_to_r1cs: String::from("https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.r1cs"),
            path_to_wasm: String::from("https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.wasm"),
            path_to_zkey: String::from("https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.zkey"),
            builder_params: vec![
                String::from("x1"),
                String::from("y1"),
                String::from("x2"),
                String::from("y2"),
                String::from("r"),
                String::from("distMax"),
                String::from("PLANETHASH_KEY"),
                String::from("SPACETYPE_KEY"),
                String::from("SCALE"),
                String::from("xMirror"),
                String::from("yMirror"),
            ],
        };
        let response = client.post("/v1/prover").json(&prover).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let mut proof_request: HashMap<String, u64> = HashMap::new();

        proof_request.insert(String::from("x1"), 100);
        proof_request.insert(String::from("y1"), 100);
        proof_request.insert(String::from("x2"), 120);
        proof_request.insert(String::from("y2"), 120);
        proof_request.insert(String::from("r"), 8000);
        proof_request.insert(String::from("distMax"), 0);
        proof_request.insert(String::from("PLANETHASH_KEY"), 1729);
        proof_request.insert(String::from("SPACETYPE_KEY"), 1730);
        proof_request.insert(String::from("xMirror"), false as u64);
        proof_request.insert(String::from("SCALE"), 16384);
        proof_request.insert(String::from("yMirror"), false as u64);

        let response = client
            .post("/v1/prove/test")
            .json(&proof_request)
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }
    #[test]
    fn test_missing_proof_arg() {
        let rocket_instance = rocket();
        let client = Client::tracked(rocket_instance).expect("valid rocket instance");

        let prover = ProverConfig {
            name: String::from("test"),
            version: String::from("0.0.1"),
            path_to_r1cs: String::from("https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.r1cs"),
            path_to_wasm: String::from("https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.wasm"),
            path_to_zkey: String::from("https://unpkg.com/@darkforest_eth/snarks@6.6.6/move.zkey"),
            builder_params: vec![
                String::from("x1"),
                String::from("y1"),
                String::from("x2"),
                String::from("y2"),
                String::from("r"),
                String::from("distMax"),
                String::from("PLANETHASH_KEY"),
                String::from("SPACETYPE_KEY"),
                String::from("SCALE"),
                String::from("xMirror"),
                String::from("yMirror"),
            ],
        };
        let response = client.post("/v1/prover").json(&prover).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let mut proof_request: HashMap<String, u64> = HashMap::new();

        proof_request.insert(String::from("x1"), 100);
        proof_request.insert(String::from("y1"), 100);
        proof_request.insert(String::from("y2"), 120);
        proof_request.insert(String::from("r"), 8000);
        proof_request.insert(String::from("distMax"), 0);
        proof_request.insert(String::from("PLANETHASH_KEY"), 1729);
        proof_request.insert(String::from("SPACETYPE_KEY"), 1730);
        proof_request.insert(String::from("xMirror"), false as u64);
        proof_request.insert(String::from("SCALE"), 16384);
        proof_request.insert(String::from("yMirror"), false as u64);

        let response = client
            .post("/v1/prove/test")
            .json(&proof_request)
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(
            response.into_string().unwrap(),
            String::from("x2 is missing from your inputs")
        )
    }
}
