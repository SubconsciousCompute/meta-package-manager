use std::str::FromStr;

use clap::{Parser, Subcommand};

use crate::{Package, PackageManager};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A generic package manager.",
    long_about = "A generic package manager for interfacing with multiple distro and platform specific package managers."
)]

/// Cli for PackageManager.
///
/// It is public because other tools can use this interface to pass the command
/// line args.
pub struct Cli {
    #[command(subcommand)]
    command: MpmPackageManagerCommands,

    /// Optionally specify a package manager that you want to use. If not given,
    /// mpm will search for default package manager on this system.
    #[arg(long, short)]
    manager: Option<crate::common::AvailablePackageManager>,

    // TODO: See issue #33
    // /// Set interactive mode
    // #[arg(long, short, default_value_t = false)]
    // interactive: bool,
    /// Set output to be in json format.
    #[arg(long, default_value_t = false)]
    json: bool,
}

#[derive(Subcommand)]
pub enum MpmPackageManagerCommands {
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

/// Function that handles the parsed CLI arguments in one place
pub fn execute(args: Cli) -> anyhow::Result<()> {
    let mpm = if let Some(manager) = args.manager {
        crate::MetaPackageManager::new_if_available(manager)?
    } else {
        crate::MetaPackageManager::new_default()?
    };

    match args.command {
        MpmPackageManagerCommands::Managers => crate::print::print_managers(),
        MpmPackageManagerCommands::Search { string } => {
            let pkgs = mpm.search(&string);
            print_pkgs(&pkgs, args.json)?;
        }
        MpmPackageManagerCommands::List => {
            let pkgs = mpm.list_installed();
            print_pkgs(&pkgs, args.json)?;
        }
        MpmPackageManagerCommands::Install { packages } => {
            for pkg in packages {
                let pkg_path = std::path::PathBuf::from(&pkg);
                let s = if pkg_path.is_file() {
                    mpm.install(&pkg_path)
                } else {
                    mpm.install(pkg.as_str())
                };
                anyhow::ensure!(s.success(), "Failed to install {pkg}");
            }
        }
        MpmPackageManagerCommands::Uninstall { packages } => {
            for pkg in packages {
                let s = mpm.uninstall(Package::from_str(&pkg)?);
                anyhow::ensure!(s.success(), "Failed to uninstall pacakge {pkg}");
            }
        }

        MpmPackageManagerCommands::Update { packages, all } => {
            if all {
                mpm.update_all();
            } else {
                for pkg in packages {
                    let s = mpm.update(Package::from_str(&pkg)?);
                    anyhow::ensure!(s.success(), "Failed to update pacakge {pkg}");
                }
            }
        }
        MpmPackageManagerCommands::Repo { repo } => {
            mpm.add_repo(&repo)?;
        }
        MpmPackageManagerCommands::Sync => {
            let s = mpm.sync();
            anyhow::ensure!(s.success(), "Failed to sync repositories");
        }
    };

    Ok(())
}

/// Print packages
fn print_pkgs(pkgs: &[Package], json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(pkgs)?);
    } else {
        println!("{}", tabled::Table::new(pkgs));
    }
    Ok(())
}
