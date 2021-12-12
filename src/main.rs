use rocket::http::Status;
mod storage;
use rocket::serde::json::Json;

#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[post("/prover", format = "json", data = "<prover>")]
pub async fn add_prover_handler(
    db: &rocket::State<storage::Db>,
    prover: Json<storage::ProverConfig>,
) -> Status {
    let mut db = db.lock().await;
    let prover = prover.into_inner();
    let wasm_url = prover.path_to_wasm.clone();
    db.insert(prover.name.clone(), prover);

    let resp = match reqwest::get(wasm_url).await {
        Ok(res) => res,
        Err(_) => return Status::BadRequest,
    };
    println!("{:?} {:?}", resp.status(), resp.text().await);

    return Status::Accepted;
}
#[get("/prover")]
pub async fn list_provers_handler(
    db: &rocket::State<storage::Db>,
) -> Option<Json<Vec<storage::ProverConfig>>> {
    let prover_hm = db.lock().await;
    let provers: Vec<storage::ProverConfig> = prover_hm.values().cloned().collect();
    println!("database {:?}", provers);
    return Some(Json(provers));
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(storage::init_storage())
        .mount("/", routes![index])
        .mount("/v1/", routes![add_prover_handler, list_provers_handler])
}
