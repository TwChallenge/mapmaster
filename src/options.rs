use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Options {
    #[structopt(short, long)]
    pub base: Option<PathBuf>,

    #[structopt(short, long)]
    pub apikeys: Option<PathBuf>,

    #[structopt(short, long)]
    pub dev: bool,
}
