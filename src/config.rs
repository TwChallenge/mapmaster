use std::path::PathBuf;

pub struct Config {
    pub apikeys: Vec<String>,
    pub base: PathBuf,
    pub dev: bool,
}
