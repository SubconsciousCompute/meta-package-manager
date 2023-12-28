use clap::Parser;
use utils::parser::Cli;

mod utils;

fn main() {
    if let Err(err) = utils::execute(Cli::parse()) {
        utils::print::log_error(err);
        std::process::exit(1);
    }
}
