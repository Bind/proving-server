use crate::errors::ProvingServerError;
use crate::types::proof::ProofInputs;
use crate::types::reqres::ProverConfigRequest;
use rusqlite::types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{params, Connection, Result};
pub trait CRUD {
    fn create(&mut self, conn: &Connection) -> Result<usize, rusqlite::Error>;
    fn get(id: i64, conn: &Connection) -> Result<Self, rusqlite::Error>
    where
        Self: Sized;
    fn update(&mut self, conn: &Connection) -> Result<usize, rusqlite::Error>;
    fn delete(&self, conn: &Connection) -> Result<usize, rusqlite::Error>;
}

#[derive(Debug, Clone)]
pub struct ProverConfig {
    pub id: Option<i64>,
    pub name: String,
    pub version: String,
    pub path_to_wasm: String,
    pub path_to_zkey: String,
    pub path_to_r1cs: String,
    pub builder_params: Vec<String>,
}

impl From<ProverConfigRequest> for ProverConfig {
    fn from(r: ProverConfigRequest) -> ProverConfig {
        ProverConfig {
            id: None,
            name: r.name,
            version: r.version,
            path_to_wasm: r.path_to_wasm,
            path_to_zkey: r.path_to_zkey,
            path_to_r1cs: r.path_to_r1cs,
            builder_params: r.builder_params.clone(),
        }
    }
}
impl ProverConfig {
    pub fn validate_inputs(&self, inputs: &ProofInputs) -> Result<bool, ProvingServerError> {
        for param in &self.builder_params {
            if !inputs.contains_key(&param.clone()) {
                return Err(ProvingServerError::BadProofInputsError {
                    message: String::from(format!("{}", param.clone())),
                });
            }
        }
        return Ok(true);
    }
    pub fn get_by_name(name: String, conn: &Connection) -> Result<ProverConfig, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, version, path_to_wasm, path_to_zkey,path_to_r1cs FROM Prover where name = ?1"
        )?;

        let mut query_map_stmt =
            conn.prepare("SELECT name, prover FROM builder_params where prover = ?1")?;
        let mut prover_iter = stmt.query_map(params![name], |row| {
            let id = row.get(0)?;
            let b_params: Vec<String> = query_map_stmt
                .query_map(params![id], |param_row| {
                    return Ok(param_row.get(0).unwrap());
                })
                .unwrap()
                .map(|r| r.unwrap())
                .collect();

            Ok(ProverConfig {
                id: id,
                name: row.get(1)?,
                version: row.get(2)?,
                path_to_wasm: row.get(3)?,
                path_to_zkey: row.get(4)?,
                path_to_r1cs: row.get(5)?,
                builder_params: b_params.clone(),
            })
        })?;
        // Gross
        return Ok(prover_iter.next().unwrap().unwrap());
    }
}

impl CRUD for ProverConfig {
    fn create(&mut self, conn: &Connection) -> Result<usize, rusqlite::Error> {
        let initial =  conn.execute(
            "insert into Prover (name, version, path_to_wasm, path_to_zkey, path_to_r1cs) values (?1,?2,?3, ?4, ?5) ",
            params![self.name,self.version, self.path_to_wasm, self.path_to_zkey, self.path_to_r1cs],
        );
        let prover_id = conn.last_insert_rowid().clone();
        self.id = Some(prover_id.clone());
        for param in &self.builder_params {
            match conn.execute(
                "insert into builder_params (name, prover) values (?1, ?2)",
                params![param, prover_id],
            ) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }
        return initial;
    }
    fn get(id: i64, conn: &Connection) -> Result<ProverConfig, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, version, path_to_wasm, path_to_zkey,path_to_r1cs FROM Prover where id = ?1"
        )?;

        let mut query_map_stmt =
            conn.prepare("SELECT name, prover FROM builder_params where prover = ?1")?;
        let mut prover_iter = stmt.query_map(params![id], |row| {
            let b_params: Vec<String> = query_map_stmt
                .query_map(params![id], |param_row| {
                    return Ok(param_row.get(0).unwrap());
                })
                .unwrap()
                .map(|r| r.unwrap())
                .collect();

            Ok(ProverConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                path_to_wasm: row.get(3)?,
                path_to_zkey: row.get(4)?,
                path_to_r1cs: row.get(5)?,
                builder_params: b_params.clone(),
            })
        })?;
        // Gross
        return Ok(prover_iter.next().unwrap().unwrap());
    }
    fn update(&mut self, conn: &Connection) -> Result<usize, rusqlite::Error> {
        todo!()
    }
    fn delete(&self, conn: &Connection) -> Result<usize, rusqlite::Error> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JobStatus {
    PENDING = 0,
    QUEUED = 1,
    PROCESSING = 2,
    READY = 3,
}

impl FromSql for JobStatus {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        let inter = value.as_i64().unwrap();
        match inter {
            x if x == JobStatus::PENDING as i64 => Ok(JobStatus::PENDING),
            x if x == JobStatus::QUEUED as i64 => Ok(JobStatus::QUEUED),
            x if x == JobStatus::PROCESSING as i64 => Ok(JobStatus::PROCESSING),
            x if x == JobStatus::READY as i64 => Ok(JobStatus::READY),
            _ => return Err(FromSqlError::OutOfRange(inter)),
        }
    }
}
impl ToSql for JobStatus {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::from(*self as i64))
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    pub id: Option<i64>,
    pub status: JobStatus,
    pub message: String,
    pub prover: i64,
}

impl CRUD for Job {
    fn create(&mut self, conn: &Connection) -> Result<usize, rusqlite::Error> {
        let init = conn.execute(
            "insert into job (status, message, prover) values (?1,?2,?3) ",
            params![self.status, self.message, self.prover],
        );
        let prover_id = conn.last_insert_rowid().clone();
        self.id = Some(prover_id.clone());
        return init;
    }
    fn get(id: i64, conn: &Connection) -> Result<Job, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT id, status, message, prover FROM job where id = ?1")?;
        let jobs: Vec<Job> = stmt
            .query_map(params![id], |row| {
                return Ok(Job {
                    id: row.get(0)?,
                    status: row.get(1)?,
                    message: row.get(2)?,
                    prover: row.get(3)?,
                });
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        // also gross
        Ok(jobs.get(0).unwrap().clone())
    }
    fn update(&mut self, conn: &Connection) -> Result<usize, rusqlite::Error> {
        let init = conn.execute(
            "update job set status = ?2, message = ?3 where id = ?1",
            params![self.id, self.status, self.message],
        );
        let prover_id = conn.last_insert_rowid().clone();
        self.id = Some(prover_id.clone());
        return init;
    }
    fn delete(&self, conn: &Connection) -> Result<usize, rusqlite::Error> {
        todo!()
    }
}

#[tokio::test]
async fn create_job() {
    use crate::test::fixtures;
    let conn = fixtures::setup_db().await;
    let mut prover = fixtures::df_prover_config();
    ProverConfig::create(&mut prover, &conn).unwrap();

    let mut job = Job {
        id: None,
        status: JobStatus::PENDING,
        message: String::from("test initiatization"),
        prover: prover.id.unwrap(),
    };
    job.create(&conn).unwrap();
    let j2 = Job::get(job.id.unwrap(), &conn).unwrap();
    assert_eq!(j2, job);
}
