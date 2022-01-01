mod db;
mod errors;
mod models;
mod prover;
mod routes;
mod test;
mod types;
mod utils;
mod worker;
use std::sync::mpsc;

extern crate dotenv;

#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    utils::load_environment_variables();
    let (tx, rx) = mpsc::sync_channel(1);
    let config = utils::init_config();
    let conn: types::Db = db::init_async_database(config.clone()).unwrap();
    let provers = utils::init_provers();

    // Create pointers for thread to reference
    let t_conn = conn.clone();
    let t_provers = provers.clone();
    tokio::spawn(async move { worker::worker(t_conn, config, t_provers, rx).await });

    rocket::build()
        .manage(types::JobSender(tx))
        .manage(conn)
        .manage(utils::init_async_config())
        .manage(provers)
        .mount("/", routes![routes::index])
        .mount(
            "/v1/",
            routes![
                routes::add_prover_handler,
                routes::list_provers_handler,
                routes::execute_prover,
                routes::get_prover
            ],
        )
}

/**
 *
 *
 * TESTS
 *
 */

#[cfg(test)]
mod main_tests {
    use super::rocket;
    use crate::models::JobStatus;
    use crate::test::fixtures;
    use crate::types::reqres::JobResponse;
    use rocket::http::Status;
    use rocket::local::asynchronous::Client as AsyncClient;
    use rocket::local::blocking::Client;
    use std::{thread, time};
    pub fn wait_for_job_status(
        client: &Client,
        prover_name: String,
        prover_version: String,
        status: JobStatus,
    ) {
        let mut curr_status = JobStatus::Pending;

        while status != curr_status {
            let response = client
                .get(format!("/v1/prover/{}/{}", prover_name, prover_version))
                .dispatch();
            let job_status: JobResponse =
                rocket::serde::json::from_str(&response.into_string().unwrap()).unwrap();
            curr_status = job_status.status;
        }
        return;
    }
    pub async fn async_wait_for_job_status(
        client: &AsyncClient,
        prover_name: String,
        prover_version: String,
        status: JobStatus,
    ) {
        let mut curr_status = JobStatus::Pending;
        let five = time::Duration::from_secs(5);
        while status != curr_status {
            thread::sleep(five);
            let response = client
                .get(format!("/v1/prover/{}/{}", prover_name, prover_version))
                .dispatch()
                .await;
            let job_status: JobResponse =
                rocket::serde::json::from_str(&response.into_string().await.unwrap()).unwrap();
            println!("{:?}", job_status);
            curr_status = job_status.status;
        }
        return;
    }
    #[rocket::async_test]
    async fn int_add_prover_route() {
        let rocket_instance = rocket();
        let client = AsyncClient::tracked(rocket_instance).await.unwrap();
        let prover = crate::test::fixtures::df_prover_config_request();
        let response = client.post("/v1/prover").json(&prover).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }
    #[rocket::async_test]
    async fn int_proof_generation() {
        use rocket::local::asynchronous::Client;
        let rocket_instance = rocket();
        let client = Client::tracked(rocket_instance).await.unwrap();
        let prover = crate::test::fixtures::df_prover_config_request();
        let response = client.post("/v1/prover").json(&prover).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        async_wait_for_job_status(
            &client,
            prover.name.clone(),
            prover.version.clone(),
            JobStatus::Ready,
        )
        .await;
        let proof_request = fixtures::df_proof_request();
        let response = client
            .post(format!(
                "/v1/prove/{}/{}",
                prover.name.clone(),
                prover.version.clone()
            ))
            .json(&proof_request)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::Ok);

        let response = client.post("/v1/prover").json(&prover).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
    }

    #[test]
    #[should_panic]
    fn int_bad_proof_generation() {
        let rocket_instance = rocket();
        let client = Client::tracked(rocket_instance).expect("valid rocket instance");

        let prover = fixtures::df_prover_config_request();
        let response = client.post("/v1/prover").json(&prover).dispatch();
        assert_eq!(response.status(), Status::Ok);
        let proof_request = fixtures::df_proof_request();
        wait_for_job_status(
            &client,
            prover.name.clone(),
            prover.version.clone(),
            JobStatus::Ready,
        );
        let response = client
            .post(format!(
                "/v1/prove/{}/{}",
                prover.name.clone(),
                prover.version.clone()
            ))
            .json(&proof_request)
            .dispatch();
        assert_eq!(response.status(), Status::BadRequest);
    }
    #[rocket::async_test]
    async fn int_missing_proof_arg() {
        let rocket_instance = rocket();
        let client = AsyncClient::tracked(rocket_instance).await.unwrap();

        let prover = fixtures::df_prover_config_request();
        let response = client.post("/v1/prover").json(&prover).dispatch().await;
        assert_eq!(response.status(), Status::Ok);
        async_wait_for_job_status(
            &client,
            prover.name.clone(),
            prover.version.clone(),
            JobStatus::Ready,
        )
        .await;
        let mut proof_request = fixtures::df_proof_request();
        proof_request.remove("x2");

        let response = client
            .post(format!(
                "/v1/prove/{}/{}",
                prover.name.clone(),
                prover.version.clone()
            ))
            .json(&proof_request)
            .dispatch()
            .await;
        assert_eq!(response.status(), Status::BadRequest);
        assert_eq!(
            response.into_string().await.unwrap(),
            String::from("x2 is missing from your inputs")
        )
    }
}
