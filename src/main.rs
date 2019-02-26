use failure::Error;
use structopt::StructOpt;

use count::{run, Config};

fn main() -> Result<(), Error> {
    let config = Config::from_args();

    run(config)
}
