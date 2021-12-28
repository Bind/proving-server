use crate::errors::ProvingServerError;
use crate::prover;
use crate::types::{
    to_eth_type, Abc, CircuitProver, Config, Db, ProofInputs, ProverConfig, Provers,
};
use crate::utils::files::{fetch_file, get_r1cs_path, get_wasm_path, get_zkey_path};
use ark_circom::ethereum::Proof;
use rocket::http::Status;
use rocket::serde::json::Json;

#[get("/")]
pub fn index() -> &'static str {
    "Hello, world!"
}

#[get("/prover")]
pub async fn list_provers_handler(db: &rocket::State<Db>) -> Option<Json<Vec<ProverConfig>>> {
    let prover_hm = db.lock().await;
    let provers: Vec<ProverConfig> = prover_hm.values().cloned().collect();
    return Some(Json(provers));
}

#[post("/prove/<prover_name>", data = "<inputs>")]
pub async fn execute_prover(
    prover_storage: &rocket::State<Provers>,
    prover_cfg: &rocket::State<Db>,
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

#[post("/prover", format = "json", data = "<prover>")]
pub async fn add_prover_handler(
    db: &rocket::State<Db>,
    prover_storage: &rocket::State<Provers>,
    config: &rocket::State<Config>,
    prover: Json<ProverConfig>,
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
    let p = CircuitProver::new_path(zkey_path, wasm_path, r1cs_path).unwrap();

    let mut prover_storage = prover_storage.lock().await;
    prover_storage.insert(prover.name.clone(), p);
    return Status::Ok;
}
