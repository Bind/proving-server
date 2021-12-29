use crate::types::{Config, DatabaseMode};
use rusqlite::{params, Connection, Result};

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
        version TEXT NOT NULL,
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
        message TEXT not null,
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

async fn setup() -> Connection {
    use crate::storage::init_async_config;
    use crate::utils::load_environment_variables;

    load_environment_variables();
    let config = init_async_config();
    let conn = init_database(config).await.unwrap();
    let conn = init_tables(conn).unwrap();
    return conn;
}

#[tokio::test]
async fn test_table_init() -> Result<()> {
    let conn = setup().await;
    let mut statement =
        conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='prover'")?;
    let id_iter = statement.query_map([], |row| Ok(Table { id: row.get(0)? }))?;
    for row in id_iter {
        println!("{:?}", row.unwrap());
    }
    Ok(())
}

#[tokio::test]
async fn test_insert_into_prover() -> Result<()> {
    use crate::types::ProverConfig;
    use rusqlite::params;
    let conn = setup().await;
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
    conn.execute(
        "insert into Prover (name, version, path_to_wasm, path_to_zkey, path_to_r1cs) values (?1,?2,?3, ?4, ?5) ",
        params![prover.name,prover.version, prover.path_to_wasm, prover.path_to_zkey, prover.path_to_r1cs],
    );
    let mut stmt = conn
        .prepare("SELECT id, name, version, path_to_wasm, path_to_zkey,path_to_r1cs FROM Prover")?;
    let prover_iter = stmt.query_map([], |row| {
        Ok(ProverConfig {
            name: row.get(1)?,
            version: row.get(2)?,
            path_to_wasm: row.get(3)?,
            path_to_zkey: row.get(4)?,
            path_to_r1cs: row.get(5)?,
            builder_params: vec![],
        })
    })?;
    let provers: Vec<ProverConfig> = prover_iter.map(|r| r.unwrap()).collect();
    assert_eq!(provers.len(), 1);
    Ok(())
}
