use crate::storage::Config;
use crate::types::DatabaseMode;
use rusqlite::{Connection, Result};

#[derive(Debug)]
pub struct ProverConfig {
    id: String,
    pub name: String,
    pub version: String,
    pub path_to_wasm: String,
    pub path_to_zkey: String,
    pub path_to_r1cs: String,
    pub builder_params: Vec<String>,
}

pub async fn init_database(config: Config) -> Result<Connection> {
    let config = config.lock().await;
    let conn = match &config.db_config {
        DatabaseMode::Memory => Connection::open_in_memory()?,
        DatabaseMode::File { path_to_file } => Connection::open(path_to_file.clone())?,
    };

    return Ok(conn);
}

pub fn init_tables(conn: Connection) -> Result<Connection> {
    conn.execute(
        "
    CREATE TABLE prover (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        path_to_wasm  TEXT NOT NULL,
        path_to_zkey TEXT NOT NULL,
        path_to_r1cs TEXT NOT NULL
    )",
        [],
    )?;
    conn.execute(
        "
    CREATE TABLE builder_params (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        prover INTEGER,
        FOREIGN KEY(prover) REFERENCES prover(id)

    )
    ",
        [],
    )?;
    conn.execute(
        "
    CREATE TABLE job (
        id INTEGER PRIMARY KEY,
        status INTEGER not null,
        prover INTEGER,
        FOREIGN KEY(prover) REFERENCES prover(id)
    )
    ",
        [],
    )?;
    Ok(conn)
}
#[derive(Debug)]
pub struct Table {
    id: String,
}

mod test {
    use crate::storage;
    use crate::utils;
    use rusqlite::{params, Connection, Result};

    #[tokio::test]
    async fn test_table_init() -> Result<()> {
        utils::load_environment_variables();
        let config = storage::init_async_config();
        let conn = super::init_database(config).await.unwrap();

        let mut statement =
            conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='prover'")?;
        let id_iter = statement.query_map([], |row| Ok(super::Table { id: row.get(0)? }))?;
        for row in id_iter {
            println!("{:?}", row.unwrap());
        }
        Ok(())
    }
}
