use crate::storage::{Db, ProverConfig};
use warp::{self, http::StatusCode, Future};

pub async fn add_prover_handler(new_prover: ProverConfig, db: Db) -> impl warp::Reply {
    println!("stub");
    return warp::reply::reply();
}
