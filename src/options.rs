use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Options {
    /// The folder to use as a base for all test maps.
    #[structopt(
        short,
        long,
        name = "test directory",
        default_value = "./maps/test"
    )]
    pub test_maps: PathBuf,

    /// The folder to use as a base for all published maps.
    #[structopt(short, long, name = "directory", default_value = "./maps")]
    pub published_maps: PathBuf,

    /// The file which contains the API keys for access.
    #[structopt(
        short,
        long,
        name = "api text file",
        default_value = "./apikeys"
    )]
    pub apikeys: PathBuf,

    /// Enables developer mode. With developer mode enabled, you wont need an api key to call the
    /// api.
    #[structopt(short, long)]
    pub dev: bool,
}
