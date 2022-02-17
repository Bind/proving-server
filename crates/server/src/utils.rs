use crate::types::{Config, DatabaseMode, EnvConfig};
use dotenv::from_filename;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
pub mod files {
    use crate::models::ProverConfig;
    use crate::types::EnvConfig;
    use rocket::http::Status;
    use std::fs::create_dir_all as createDir;
    use std::fs::File;
    use std::io::copy;
    use std::io::ErrorKind;
    use std::path::{Path, PathBuf};
    pub fn create_dir(archive_dir: &Path) {
        match createDir(&archive_dir) {
            Err(why) => match why.kind() {
                ErrorKind::AlreadyExists => {}
                other_error => {
                    println! {"Current dir {:?}", std::env::current_dir().unwrap()}
                    println!("Looking for {:?}", archive_dir);
                    panic!("! {:?}", other_error)
                }
            },
            Ok(_) => {
                println!("creating archive dir at {:?}", archive_dir)
            }
        }
    }
    pub fn get_zkey_path(prover: &ProverConfig, config: EnvConfig) -> PathBuf {
        let mut path = get_path_from_prover(prover, config).unwrap();
        path.set_extension("zkey");
        path
    }
    pub fn get_wasm_path(prover: &ProverConfig, config: EnvConfig) -> PathBuf {
        let mut path = get_path_from_prover(prover, config).unwrap();
        path.set_extension("wasm");
        path
    }
    pub fn get_r1cs_path(prover: &ProverConfig, config: EnvConfig) -> PathBuf {
        let mut path = get_path_from_prover(prover, config).unwrap();
        path.set_extension("r1cs");
        path
    }

    pub fn get_path_from_prover(
        prover: &ProverConfig,
        config: EnvConfig,
    ) -> Result<PathBuf, std::io::Error> {
        let mut path = PathBuf::from(config.zk_file_path);
        path = path.join(prover.version.clone());
        create_dir(&path);
        path = path.join(prover.name.clone());
        Ok(path)
    }

    pub async fn fetch_file(path: PathBuf, url: String) -> Status {
        let resp = match reqwest::get(url.clone()).await {
            Ok(res) => res,
            Err(_) => return Status::BadRequest,
        };
        println!("{:?} {:?}", resp.status(), url.clone());
        let mut dest = File::create(path.clone()).unwrap();
        let content = resp.bytes().await.unwrap();
        println!("copying {:?} to {:?}", url, path.clone().as_os_str());
        copy(&mut content.as_ref(), &mut dest).unwrap();
        Status::Accepted
    }
}
pub fn load_environment_variables() {
    if cfg!(test) {
        from_filename(".env.test").ok();
    } else {
        from_filename(".env").ok();
    }
    dotenv::dotenv().ok();
}

pub fn init_provers() -> crate::types::proof::Provers {
    Arc::new(Mutex::new(HashMap::new()))
}
pub fn init_config() -> EnvConfig {
    let zk_file_path = env::var("ZK_FILE_PATH").unwrap();
    let db_config = match env::var("DB_FILE_PATH") {
        Ok(string) => DatabaseMode::File {
            path_to_file: string,
        },
        Err(_) => DatabaseMode::Memory,
    };
    let port = match env::var("PORT") {
        Ok(num) => num.parse::<i32>().unwrap(),
        Err(_) => 8000,
    };

    EnvConfig {
        zk_file_path,
        db_config,
        port,
    }
}
pub fn init_async_config() -> Config {
    let conf = init_config();
    Arc::new(Mutex::new(conf))
}
