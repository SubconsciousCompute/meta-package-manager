//! Simple Usage.

use mpm::{MetaPackageManager, PackageManager};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let manager = MetaPackageManager::new_default().expect("brew could not be initialised");

    // most methods return `ExitStatus` which can be used to check if
    // the operation was successful
    if !manager.install("gimp").success() {
        eprintln!("Failed to install gimp");
    }

    if manager.update_all().success() {
        println!("All packages updated/upgraded");
    }

    // get packages matching search string
    let searched = manager.search("python");
    println!("Searched: {searched:#?}");

    // list installed packages
    for p in manager.list_installed() {
        println!("{p}");
    }

    Ok(())
}
