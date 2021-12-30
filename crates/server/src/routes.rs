use crate::errors::ProvingServerError;
use crate::models::{Job, JobStatus, ProverConfig, CRUD};
use crate::prover;
use crate::types::proof::{to_eth_type, Abc, ProofInputs, Provers};
use crate::types::reqres::ProverConfigRequest;
use crate::types::{Config, Db, JobSender};
use ark_circom::ethereum::Proof;
use rocket::http::Status;
use rocket::serde::json::Json;

#[get("/")]
pub fn index() -> &'static str {
    "Hello, world!"
}

#[get("/prover")]
pub async fn list_provers_handler(
    db: &rocket::State<Db>,
) -> Option<Json<Vec<ProverConfigRequest>>> {
    todo!();
}

#[post("/prove/<prover_name>", data = "<inputs>")]
pub async fn execute_prover(
    prover_storage: &rocket::State<Provers>,
    db: &rocket::State<Db>,
    prover_name: &str,
    inputs: Json<ProofInputs>,
) -> Result<Json<Abc>, ProvingServerError> {
    println!("fetching prover");
    let prover_storage = prover_storage.lock().await;
    let p = prover_storage.get(prover_name).unwrap();
    let db = db.lock().await;
    println!("fetching prover config");
    let prover = ProverConfig::get_by_name(String::from(prover_name), &db).unwrap();

    let proof_inputs = inputs.into_inner();
    match prover.validate_inputs(&proof_inputs) {
        Err(error) => return Err(error),
        _ => (),
    }

    println!("generating circuit");
    let circuit = prover::build_inputs(p.clone(), prover.clone(), proof_inputs);
    let (proof, _) = prover::prove(circuit, &p.params).unwrap();

    return Ok(Json(to_eth_type(Proof::from(proof))));
}

#[post("/prover", format = "json", data = "<prover>")]
pub async fn add_prover_handler(
    db: &rocket::State<Db>,
    prover: Json<ProverConfigRequest>,
    queue: &rocket::State<crate::types::JobSender>,
) -> Status {
    let db = db.lock().await;
    let prover = prover.into_inner();
    let mut p = ProverConfig::from(prover);
    p.create(&db);
    let j = &mut Job {
        id: None,
        status: JobStatus::PENDING,
        prover: p.id.unwrap(),
        message: String::from("asdf"),
    };
    Job::create(j, &db).unwrap();

    queue.0.try_send(j.id.unwrap());
    return Status::Ok;
}
