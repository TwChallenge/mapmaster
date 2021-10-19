use std::path::PathBuf;

pub struct Config {
    pub apikeys: Vec<String>,
    pub test_map_folder: PathBuf,
    pub public_map_folder: PathBuf,
    pub dev: bool,
}
