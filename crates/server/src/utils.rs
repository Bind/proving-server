use dotenv::from_filename;

pub mod files {
    use std::fs::create_dir as createDir;
    use std::io::ErrorKind;
    use std::path::PathBuf;

    pub fn create_dir(archive_dir: &PathBuf) {
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
}
pub fn load_environment_variables() {
    if cfg!(test) {
        from_filename(".env.test").ok();
    } else {
        from_filename(".env").ok();
    }
    dotenv::dotenv().ok();
}
