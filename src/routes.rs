use crate::handler::add_prover_handler;
use crate::storage::{Db, ProverConfig};

use std::convert::Infallible;
use warp::{self, Filter};

pub fn all_routes(
    db: Db,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    return add_prover(db.clone());
}

fn add_prover(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    return warp::path!("prover")
        .and(warp::post())
        .and(json_body())
        .and(with_storage(db))
        .then(add_prover_handler);
}

pub fn with_storage(db: Db) -> impl Filter<Extract = (Db,), Error = Infallible> + Clone {
    return warp::any().map(move || db.clone());
}

fn json_body() -> impl Filter<Extract = (ProverConfig,), Error = warp::Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}
