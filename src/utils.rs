pub mod files {
    use std::fs::create_dir as createDir;
    use std::io::ErrorKind;
    use std::path::PathBuf;

    pub fn create_dir(archive_dir: &PathBuf) {
        match createDir(&archive_dir) {
            Err(why) => match why.kind() {
                ErrorKind::AlreadyExists => {}
                other_error => {
                    panic!("! {:?}", other_error)
                }
            },
            Ok(_) => {
                println!("creating archive dir at {:?}", archive_dir)
            }
        }
    }
}
