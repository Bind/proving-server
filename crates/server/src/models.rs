use crate::errors::ProvingServerError;
use crate::types::proof::ProofInputs;
use crate::types::reqres::ProverConfigRequest;
use rocket::serde::{Deserialize, Serialize};
use rusqlite::types::{FromSql, FromSqlError, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{params, Connection, Result};
pub trait Crud {
    fn create(&mut self, conn: &Connection) -> Result<usize, rusqlite::Error>;
    fn get(id: i64, conn: &Connection) -> Result<Self, rusqlite::Error>
    where
        Self: Sized;
    fn update(&mut self, conn: &Connection) -> Result<usize, rusqlite::Error>;
    fn delete(&self, conn: &Connection) -> Result<usize, rusqlite::Error>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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
            builder_params: r.builder_params,
        }
    }
}
impl ProverConfig {
    pub fn validate_inputs(&self, inputs: &ProofInputs) -> Result<bool, ProvingServerError> {
        for param in &self.builder_params {
            if !inputs.contains_key(&param.clone()) {
                return Err(ProvingServerError::BadProofInputsError {
                    message: param.clone(),
                });
            }
        }
        Ok(true)
    }
    pub fn get_builder_params(id: i64, conn: &Connection) -> Result<Vec<String>, rusqlite::Error> {
        let mut query_map_stmt =
            conn.prepare("SELECT name, prover FROM builder_params where prover = ?1")?;
        let res: Vec<String> = query_map_stmt
            .query_map(params![id], |row| Ok(row.get(0).unwrap()))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        Ok(res)
    }

    pub fn get_by_name_and_version(
        name: String,
        version: String,
        conn: &Connection,
    ) -> Result<ProverConfig, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, version, path_to_wasm, path_to_zkey,path_to_r1cs FROM Prover where name = ?1 and version =?2"
        )?;

        let mut prover_iter = stmt.query_map(params![name, version], |row| {
            let id: i64 = row.get(0).unwrap();
            let b_params = ProverConfig::get_builder_params(id, conn).unwrap();
            Ok(ProverConfig {
                id: Some(id),
                name: row.get(1)?,
                version: row.get(2)?,
                path_to_wasm: row.get(3)?,
                path_to_zkey: row.get(4)?,
                path_to_r1cs: row.get(5)?,
                builder_params: b_params,
            })
        })?;
        // Gross
        Ok(prover_iter.next().unwrap().unwrap())
    }
}

impl Crud for ProverConfig {
    fn create(&mut self, conn: &Connection) -> Result<usize, rusqlite::Error> {
        let initial =  conn.execute(
            "insert into Prover (name, version, path_to_wasm, path_to_zkey, path_to_r1cs) values (?1,?2,?3, ?4, ?5) ",
            params![self.name,self.version, self.path_to_wasm, self.path_to_zkey, self.path_to_r1cs],
        );
        let prover_id = conn.last_insert_rowid();
        self.id = Some(prover_id);
        for param in &self.builder_params {
            match conn.execute(
                "insert into builder_params (name, prover) values (?1, ?2)",
                params![param, prover_id],
            ) {
                Ok(_) => (),
                Err(e) => return Err(e),
            }
        }
        initial
    }
    fn get(id: i64, conn: &Connection) -> Result<ProverConfig, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, name, version, path_to_wasm, path_to_zkey,path_to_r1cs FROM Prover where id = ?1"
        )?;

        let mut prover_iter = stmt.query_map(params![id], |row| {
            let b_params: Vec<String> = ProverConfig::get_builder_params(id, conn).unwrap();

            Ok(ProverConfig {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                path_to_wasm: row.get(3)?,
                path_to_zkey: row.get(4)?,
                path_to_r1cs: row.get(5)?,
                builder_params: b_params,
            })
        })?;
        // Gross
        Ok(prover_iter.next().unwrap().unwrap())
    }
    fn update(&mut self, _conn: &Connection) -> Result<usize, rusqlite::Error> {
        todo!()
    }
    fn delete(&self, _conn: &Connection) -> Result<usize, rusqlite::Error> {
        todo!()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum JobStatus {
    Pending = 0,
    Queued = 1,
    Processing = 2,
    Ready = 3,
}

impl FromSql for JobStatus {
    fn column_result(value: ValueRef<'_>) -> Result<Self, FromSqlError> {
        let inter = value.as_i64().unwrap();
        match inter {
            x if x == JobStatus::Pending as i64 => Ok(JobStatus::Pending),
            x if x == JobStatus::Queued as i64 => Ok(JobStatus::Queued),
            x if x == JobStatus::Processing as i64 => Ok(JobStatus::Processing),
            x if x == JobStatus::Ready as i64 => Ok(JobStatus::Ready),
            _ => Err(FromSqlError::OutOfRange(inter)),
        }
    }
}
impl ToSql for JobStatus {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>, rusqlite::Error> {
        Ok(ToSqlOutput::from(*self as i64))
    }
}
#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct Job {
    pub id: Option<i64>,
    pub status: JobStatus,
    pub message: String,
    pub prover: i64,
}

impl Job {
    pub fn get_by_name_and_version(
        prover_name: String,
        prover_version: String,
        conn: &Connection,
    ) -> Result<Job, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT job.id, job.status, job.message, job.prover FROM job join prover on prover.id = job.prover where prover.name = ?1 and prover.version = ?2")?;
        let jobs: Vec<Job> = stmt
            .query_map(params![prover_name, prover_version], |row| {
                Ok(Job {
                    id: row.get(0)?,
                    status: row.get(1)?,
                    message: row.get(2)?,
                    prover: row.get(3)?,
                })
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        Ok(jobs.get(0).unwrap().clone())
    }
}

impl Crud for Job {
    fn create(&mut self, conn: &Connection) -> Result<usize, rusqlite::Error> {
        let init = conn.execute(
            "insert into job (status, message, prover) values (?1,?2,?3) ",
            params![self.status, self.message, self.prover],
        );
        let prover_id = conn.last_insert_rowid();
        self.id = Some(prover_id);
        init
    }
    fn get(id: i64, conn: &Connection) -> Result<Job, rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT id, status, message, prover FROM job where id = ?1")?;
        let jobs: Vec<Job> = stmt
            .query_map(params![id], |row| {
                Ok(Job {
                    id: row.get(0)?,
                    status: row.get(1)?,
                    message: row.get(2)?,
                    prover: row.get(3)?,
                })
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
        let prover_id = conn.last_insert_rowid();
        self.id = Some(prover_id);
        init
    }
    fn delete(&self, _conn: &Connection) -> Result<usize, rusqlite::Error> {
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
        status: JobStatus::Pending,
        message: String::from("test initiatization"),
        prover: prover.id.unwrap(),
    };
    job.create(&conn).unwrap();
    let j2 = Job::get(job.id.unwrap(), &conn).unwrap();
    assert_eq!(j2, job);
}
