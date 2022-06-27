use clap::Parser;
use std::process::exit;

use count::{run, Config};

fn main() {
    let config = Config::parse();

    if let Err(err) = run(config) {
        eprintln!("{}", err);
        exit(1);
    }
}
