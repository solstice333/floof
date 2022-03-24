use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "floof",
    about = "transaction processor"
)]
pub struct Args {
    #[structopt(parse(from_os_str))]
    pub tx_csv: PathBuf,

    #[structopt(short, long)] 
    pub verbose: bool,
}
