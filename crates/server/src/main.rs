mod db;
mod errors;
mod models;
mod prover;
mod routes;
mod storage;
mod test;
mod types;
mod utils;
mod worker;
use std::sync::mpsc;
use tokio;

extern crate dotenv;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    utils::load_environment_variables();
    let (tx, rx) = mpsc::sync_channel(1);
    let config = utils::init_config();
    let conn: types::Db = db::init_async_database(config.clone()).unwrap();
    let provers = utils::init_provers();

    // Create pointers for thread to reference
    let t_conn = conn.clone();
    let t_provers = provers.clone();
    tokio::spawn(async move { worker::worker(t_conn, config, t_provers, rx).await });

    rocket::build()
        .manage(types::JobSender(tx))
        .manage(conn)
        .manage(utils::init_async_config())
        .manage(provers)
        .mount("/", routes![routes::index])
        .mount(
            "/v1/",
            routes![
                routes::add_prover_handler,
                routes::list_provers_handler,
                routes::execute_prover
            ],
        )
}

/**
 *
 *
 * TESTS
 *
 */

#[cfg(test)]
mod main_tests {
    use super::rocket;
    use super::types::reqres::ProverConfigRequest;
    use rocket::http::Status;
    use rocket::local::blocking::Client;
    use std::collections::HashMap;
    fn max_distance(x1: i64, y1: i64, x2: i64, y2: i64) -> u64 {
        ((x1 - x2).pow(2) as f64 + (y1 - y2).pow(2) as f64).sqrt() as u64 + 1
    }
    #[test]
    fn add_prover_route() {
        let rocket_instance = rocket();
        let client = Client::tracked(rocket_instance).expect("valid rocket instance");

        let prover = ProverConfigRequest {
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
    }
    #[test]
    fn proof_generation() {
        let rocket_instance = rocket();
        let client = Client::tracked(rocket_instance).expect("valid rocket instance");
        let prover = ProverConfigRequest {
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
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    #[should_panic]
    fn bad_proof_generation() {
        let rocket_instance = rocket();
        let client = Client::tracked(rocket_instance).expect("valid rocket instance");

        let prover = ProverConfigRequest {
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
    fn missing_proof_arg() {
        let rocket_instance = rocket();
        let client = Client::tracked(rocket_instance).expect("valid rocket instance");

        let prover = ProverConfigRequest {
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
