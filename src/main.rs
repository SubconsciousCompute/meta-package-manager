use clap::Parser;

use mpm::utils;

fn main() {
    if let Err(err) = utils::execute(utils::parser::Cli::parse()) {
        utils::print::log_error(err);
        std::process::exit(1);
    }
}
