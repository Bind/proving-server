use crate::errors::ProvingServerError;
use crate::models::{Crud, Job, JobStatus, ProverConfig};
use crate::prover;
use crate::types::proof::{to_eth_type, Abc, Provers};
use crate::types::reqres::{JobResponse, ProofRequest, ProverConfigRequest};
use crate::types::Db;
use ark_circom::ethereum::Proof;
use rocket::http::Status;
use rocket::serde::json::Json;

#[get("/")]
pub fn index() -> &'static str {
    "Hello, world!"
}

#[get("/prover")]
pub async fn list_provers_handler(
    _db: &rocket::State<Db>,
) -> Option<Json<Vec<ProverConfigRequest>>> {
    todo!();
}

#[get("/prover/<prover_name>/<prover_version>")]
pub async fn get_prover(
    db: &rocket::State<Db>,
    prover_name: &str,
    prover_version: &str,
) -> Result<Json<JobResponse>, ProvingServerError> {
    let db = db.lock().await;

    let job =
        Job::get_by_name_and_version(String::from(prover_name), String::from(prover_version), &db)
            .unwrap();
    Ok(Json(JobResponse::from(job)))
}

#[post("/prove/<prover_name>/<prover_version>", data = "<inputs>")]
pub async fn execute_prover(
    prover_storage: &rocket::State<Provers>,
    db: &rocket::State<Db>,
    prover_name: &str,
    prover_version: &str,
    inputs: Json<ProofRequest>,
) -> Result<Json<Abc>, ProvingServerError> {
    println!("fetching prover");
    let prover_storage_guard = prover_storage.lock().await;
    let p: crate::types::proof::CircuitProver =
        prover_storage_guard.get(prover_name).unwrap().clone();
    drop(prover_storage_guard);

    let db_guard = db.lock().await;
    println!("fetching prover config");
    let prover = ProverConfig::get_by_name_and_version(
        String::from(prover_name),
        String::from(prover_version),
        &db_guard,
    )
    .unwrap();
    drop(db_guard);

    let proof_inputs = inputs.into_inner();
    if let Err(error) = prover.validate_inputs(&proof_inputs) {
        return Err(error);
    }

    println!("generating circuit");
    let circuit = prover::build_inputs(&p.clone(), prover, proof_inputs);
    let (proof, _) = prover::prove(circuit, &p.params).unwrap();

    Ok(Json(to_eth_type(Proof::from(proof))))
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
    p.create(&db).unwrap();
    let j = &mut Job {
        id: None,
        status: JobStatus::Pending,
        prover: p.id.unwrap(),
        message: format!("fetching deps for {}", p.name.clone()),
    };
    Job::create(j, &db).unwrap();

    queue.0.try_send(j.id.unwrap()).unwrap();
    Status::Ok
}
