//! Meta Package Manager (MPM) binary

use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let info = os_info::get();
    tracing::info!("Detected OS {:?}", info.os_type());

    if let Err(err) = mpm::cli::execute(mpm::cli::Cli::parse()) {
        mpm::print::log_error(err);
        std::process::exit(1);
    }
}
