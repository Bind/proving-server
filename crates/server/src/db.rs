use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};
use std::collections::HashMap;

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

fn init_tables() -> Result<Connection> {
    let conn = Connection::open_in_memory()?;
    conn.execute(
        "
    CREATE TABLE prover (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        path_to_wasm  TEXT NOT NULL,
         path_to_zkey TEXT NOT NULL,
         path_to_r1cs TEXT NOT NULL
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
    use rusqlite::{params, Connection, Result};
    #[test]
    fn test_table_init() -> Result<()> {
        let con = super::init_tables().unwrap();
        let mut statement =
            con.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='prover'")?;
        let id_iter = statement.query_map([], |row| Ok(super::Table { id: row.get(0)? }))?;
        for row in id_iter {
            println!("{:?}", row.unwrap());
        }
        Ok(())
    }
}
