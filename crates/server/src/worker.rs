use crate::models::{Job, JobStatus, ProverConfig, CRUD};
use crate::types::proof::{CircuitProver, Provers};
use crate::types::{Db, EnvConfig};
use crate::utils::files::{fetch_file, get_r1cs_path, get_wasm_path, get_zkey_path};
use std::sync::mpsc;
use std::{thread, time};

pub async fn worker(
    db: Db,
    config: EnvConfig,
    prover_storage: Provers,
    trigger: mpsc::Receiver<i64>,
) {
    loop {
        let id = trigger.recv().unwrap();
        println!("Hello! {:?}", id);

        process_job(id, &db, config.clone(), &prover_storage).await;
    }
}

async fn process_job(id: i64, db: &Db, config: EnvConfig, prover_storage: &Provers) {
    let guard = db.lock().await;
    let mut job = Job::get(id, &guard).unwrap();

    job.status = JobStatus::PROCESSING;
    job.update(&guard).unwrap();
    drop(guard);
    let guard = db.lock().await;
    let prover = ProverConfig::get(job.prover, &guard).unwrap();
    drop(guard);
    let wasm_path = get_wasm_path(&prover, config.clone());
    let zkey_path = get_zkey_path(&prover, config.clone());
    let r1cs_path = get_r1cs_path(&prover, config.clone());

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

    let guard = db.lock().await;
    job.status = JobStatus::READY;
    job.update(&guard).unwrap();
    drop(guard);
}

#[tokio::test]
async fn read_job_from_db() {
    use crate::utils::init_async_config;
    use crate::utils::load_environment_variables;
    load_environment_variables();
    let config = init_async_config();
}
