use super::Manager;
use clap::{Parser, Subcommand};
use mpm::Package;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A generic package manager.",
    long_about = "A generic package manager for interfacing with multiple distro and platform specific package managers."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    #[arg(
        long,
        short,
        help = "Specify a package manager genpack should use",
        long_help = "Optionally specify a package manager genpack should use. When no package manager is provided, a default available one is picked automatically."
    )]
    pub manager: Option<Manager>,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "List supported package managers and display their availability")]
    Managers,
    #[command(about = "Search for a given sub-string and list matching packages")]
    Search { string: String },
    #[command(about = "List all packages that are installed")]
    List,
    #[command(
        about = "Install the given package(s)",
        long_about = "Install the given package(s).\nIf a specific version of the package is desired, it can be specified using the format <package_name>@<version>.\nNote: version information is optional."
    )]
    Install {
        #[clap(required = true)]
        packages: Vec<String>,
    },
    #[command(
        about = "Uninstall the given package(s)",
        long_about = "Uninstall the given package(s).\nIf a specific version of the package is desired, it can be specified using the format <package_name>@<version>.\nNote: version information is optional."
    )]
    Uninstall {
        #[clap(required = true)]
        packages: Vec<String>,
    },
    #[command(
        about = "Add the provided third-party repo location to the package manager",
        long_about = "Provide a repo in the form of a URL or package manager specific repo format to add it to the list of repositories of the package manager"
    )]
    Repo { repo: String },
    #[command(
        about = "Updates the cached package repository data",
        long_about = "Sync the cached package repository data.\nNote: this behavior might not be consistent among package managers; when sync is not supported, the package manager might simply update itself."
    )]
    Sync,
    #[command(about = "Update/upgrade the given package(s) or (--)all of them")]
    #[group(required = true)]
    Update {
        packages: Vec<String>,
        #[arg(long, short)]
        all: bool,
    },
}

pub fn pkg_parse(pkg: &str) -> Package {
    if let Some((name, version)) = pkg.split_once('@') {
        Package::from(name).with_version(version)
    } else {
        pkg.into()
    }
}
