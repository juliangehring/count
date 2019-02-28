use std::process::exit;
use structopt::StructOpt;

use count::{run, Config};

fn main() {
    let config = Config::from_args();

    if let Err(err) = run(config) {
        eprintln!("{}", err);
        exit(1);
    }
}
