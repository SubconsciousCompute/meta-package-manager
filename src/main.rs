//! Meta Package Manager (MPM) binary

use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use mpm::utils;

fn main() {
    // elevate to sudo
    if let Err(e) = elevate::with_env(&["CARGO_", "RUST_LOG"]) {
        tracing::warn!("Failed to elevate to sudo: {e}.");
    }

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let info = os_info::get();
    tracing::info!("Detected OS {:?}", info.os_type());

    if let Err(err) = utils::execute(utils::parser::Cli::parse()) {
        utils::print::log_error(err);
        std::process::exit(1);
    }
}
