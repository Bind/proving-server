pub mod files {
    use std::fs;
    use std::fs::create_dir as createDir;
    use std::fs::{OpenOptions, ReadDir};
    use std::io;
    use std::io::ErrorKind;
    use std::io::Write;
    use std::path::{Path, PathBuf};

    pub fn check_if_file_exists(path: PathBuf) -> bool {
        return path.is_file();
    }

    pub fn overwrite_file(path: PathBuf, text: &Vec<u8>) {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .expect("unable to open");
        file.write(text).unwrap();
    }

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
    pub fn touch(path: &Path) -> io::Result<()> {
        match OpenOptions::new().create(true).write(true).open(path) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    pub fn list(target: PathBuf) -> io::Result<ReadDir> {
        return fs::read_dir(target);
    }
}
