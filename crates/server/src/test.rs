#[cfg(test)]
pub mod fixtures {
    use crate::models::ProverConfig;
    use rusqlite::Connection;

    pub async fn setup_db() -> Connection {
        use crate::db::{init_database, init_tables};
        use crate::storage::init_async_config;
        use crate::utils::load_environment_variables;

        load_environment_variables();
        let config = init_async_config();
        let conn = init_database(config).await.unwrap();
        let conn = init_tables(conn).unwrap();
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
}
