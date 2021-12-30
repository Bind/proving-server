use std::sync::mpsc;

pub fn worker(trigger: mpsc::Receiver<()>) {
    loop {
        trigger.recv().unwrap();
        println!("Hello!");
    }
}

#[tokio::test]
async fn read_job_from_db() {
    use crate::storage::init_async_config;
    use crate::utils::load_environment_variables;
    load_environment_variables();
    let config = init_async_config();
}
