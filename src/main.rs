use floof;
use args::Args;
use structopt::StructOpt;

pub mod bin_mods;
pub use bin_mods::*;

fn init_logging(verbose: bool) {
    if verbose {
        env_logger::Builder::new()
            .parse_filters("glci=trace")
            .init();
    }
}

fn main() {
    let args = Args::from_args();
    init_logging(args.verbose);
}
