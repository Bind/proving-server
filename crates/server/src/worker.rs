use crate::models::{Crud, Job, JobStatus, ProverConfig};
use crate::types::proof::{CircuitProver, Provers};
use crate::types::{Db, EnvConfig};
use crate::utils::files::{fetch_file, get_r1cs_path, get_wasm_path, get_zkey_path};
use std::sync::mpsc;

pub async fn worker(
    db: Db,
    config: EnvConfig,
    prover_storage: Provers,
    trigger: mpsc::Receiver<i64>,
) {
    loop {
        let id = trigger.recv().unwrap();
        println!("starting job for {:?}", id);
        process_job(id, &db, config.clone(), &prover_storage).await;
    }
}

async fn process_job(id: i64, db: &Db, config: EnvConfig, prover_storage: &Provers) {
    let guard = db.lock().await;
    let mut job = Job::get(id, &guard).unwrap();
    job.status = JobStatus::Processing;
    job.update(&guard).unwrap();
    drop(guard);
    let guard = db.lock().await;
    let prover = ProverConfig::get(job.prover, &guard).unwrap();
    drop(guard);
    let wasm_path = get_wasm_path(&prover, config.clone());
    let zkey_path = get_zkey_path(&prover, config.clone());
    let r1cs_path = get_r1cs_path(&prover, config.clone());

    println!(
        "writing {:?} file to {:?}",
        prover.path_to_wasm.clone(),
        wasm_path.clone()
    );
    fetch_file(wasm_path.clone(), prover.path_to_wasm.clone()).await;
    println!(
        "writing {:?} file to {:?}",
        prover.path_to_zkey.clone(),
        zkey_path.clone()
    );
    fetch_file(zkey_path.clone(), prover.path_to_zkey.clone()).await;
    println!(
        "writing {:?} file to {:?}",
        prover.path_to_r1cs.clone(),
        r1cs_path.clone()
    );
    fetch_file(r1cs_path.clone(), prover.path_to_r1cs.clone()).await;
    println!("Initializing Prover");
    let p = CircuitProver::new_path(zkey_path, wasm_path, r1cs_path).unwrap();
    let mut prover_storage = prover_storage.lock().await;
    prover_storage.insert(prover.name.clone(), p);

    let guard = db.lock().await;
    job.status = JobStatus::Ready;
    job.update(&guard).unwrap();
    drop(guard);
}

#[tokio::test]
async fn read_job_from_db() {
    use crate::utils::init_async_config;
    use crate::utils::load_environment_variables;
    load_environment_variables();
    let _config = init_async_config();
}
