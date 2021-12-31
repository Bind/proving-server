use crate::types::{DatabaseMode, Db, EnvConfig};
use rusqlite::{Connection, Result};
use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(test)]
pub async fn init_async_connection(config: Config) -> Result<Connection> {
    let config = config.lock().await;
    let conn = match &config.db_config {
        DatabaseMode::Memory => Connection::open_in_memory()?,
        DatabaseMode::File { path_to_file } => Connection::open(path_to_file.clone())?,
    };
    let conn = init_tables(conn).unwrap();
    return Ok(conn);
}

pub fn init_async_database(config: EnvConfig) -> Result<Db> {
    let conn = match &config.db_config {
        DatabaseMode::Memory => Connection::open_in_memory()?,
        DatabaseMode::File { path_to_file } => Connection::open(path_to_file.clone())?,
    };
    let conn = init_tables(conn).unwrap();
    Ok(Arc::new(Mutex::new(conn)))
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

#[tokio::test]
async fn table_init() -> Result<()> {
    let conn = crate::test::fixtures::setup_db().await;
    let mut statement =
        conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='prover'")?;
    let id_iter = statement.query_map([], |row| Ok(Table { id: row.get(0)? }))?;
    for row in id_iter {
        println!("{:?}", row.unwrap());
    }
    Ok(())
}

#[tokio::test]
async fn insert_into_prover() -> Result<()> {
    use crate::models::ProverConfig;
    use crate::test::fixtures;
    use rusqlite::params;
    let conn = fixtures::setup_db().await;
    let prover = fixtures::df_prover_config();
    conn.execute(
        "insert into Prover (name, version, path_to_wasm, path_to_zkey, path_to_r1cs) values (?1,?2,?3, ?4, ?5) ",
        params![prover.name,prover.version, prover.path_to_wasm, prover.path_to_zkey, prover.path_to_r1cs],
    ).unwrap();
    let mut stmt = conn
        .prepare("SELECT id, name, version, path_to_wasm, path_to_zkey,path_to_r1cs FROM Prover")?;
    let prover_iter = stmt.query_map([], |row| {
        Ok(ProverConfig {
            id: None,
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
