mod handler;
mod routes;
mod storage;
use std::string::String;
use std::sync::{Arc, Mutex};
use warp::{http::Method, Filter};

#[tokio::main]
async fn main() {
    let db = storage::init_storage();
    let customer_routes = routes::all_routes(db);

    warp::serve(customer_routes)
        .run(([127, 0, 0, 1], 3000))
        .await;
}
