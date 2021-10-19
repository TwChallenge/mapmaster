use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Options {
    #[structopt(short, long)]
    pub test_map_folder: Option<PathBuf>,

    #[structopt(short, long)]
    pub public_map_folder: Option<PathBuf>,

    #[structopt(short, long)]
    pub apikeys: Option<PathBuf>,

    #[structopt(short, long)]
    pub dev: bool,
}
