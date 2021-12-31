#[cfg(test)]
pub mod fixtures {
    use crate::models::ProverConfig;
    use crate::types::reqres::{ProofRequest, ProverConfigRequest};
    use rusqlite::Connection;
    fn max_distance(x1: i64, y1: i64, x2: i64, y2: i64) -> u64 {
        ((x1 - x2).pow(2) as f64 + (y1 - y2).pow(2) as f64).sqrt() as u64 + 1
    }
    pub async fn setup_db() -> Connection {
        use crate::db::{init_async_connection, init_tables};
        use crate::utils::init_async_config;
        use crate::utils::load_environment_variables;

        load_environment_variables();
        let config = init_async_config();
        let conn = init_async_connection(config).await.unwrap();
        return conn;
    }
    pub fn df_prover_config() -> ProverConfig {
        return ProverConfig {
            id: None,
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
    }
    pub fn df_prover_config_request() -> ProverConfigRequest {
        return ProverConfigRequest {
            name: String::from("move"),
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
    }
    pub fn df_proof_request() -> ProofRequest {
        let mut proof_request: ProofRequest = std::collections::HashMap::new();
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
        return proof_request;
    }
}
