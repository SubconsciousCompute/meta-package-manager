use std::str::FromStr;

use clap::{Parser, Subcommand};

use crate::{Package, PackageManager};

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A meta package manager.",
    long_about = "A meta package manager for interfacing with multiple distro and platform specific package managers."
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
    // Set interactive mode
    // #[arg(long, short, default_value_t = false)]
    // interactive: bool,
    /// Set output to be in json format.
    #[arg(long, default_value_t = false)]
    json: bool,
}

#[derive(Subcommand)]
pub enum MpmPackageManagerCommands {
    #[command(about = "List supported package managers and display their availability")]
    Managers {
        /// Install default package manager if not found e.g. choco on Windows
        /// and homebrew on osx.
        #[arg(long, default_value_t = false)]
        install_default: bool,
    },

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

    // elevate to root only for specific commands
    let requires_sudo = matches!(
        args.command,
        MpmPackageManagerCommands::Install { .. } |
        MpmPackageManagerCommands::Uninstall { .. } |
        MpmPackageManagerCommands::Update { .. } |
        MpmPackageManagerCommands::Repo { .. } |
        MpmPackageManagerCommands::Sync
    );

    if requires_sudo {
        sudo();
    }

    match args.command {
        MpmPackageManagerCommands::Managers { install_default } => {
            if install_default {
                if let Err(e) = install_default_manager() {
                    eprintln!("Failed to install default package manager: {e}");
                }
            }
            crate::print::print_managers();
        }
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

/// elevates to sudo
fn sudo() {
    #[cfg(target_os = "linux")]
    if let Err(e) = sudo::with_env(&["CARGO_", "RUST_LOG"]) {
        tracing::warn!("Failed to elevate to sudo: {e}.");
    }
}

/// install default package manager.
#[cfg(windows)]
fn install_default_manager() -> anyhow::Result<()> {
    println!("Installing choco package manager...");
    let script = "Set-ExecutionPolicy Bypass -Scope Process -Force; [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.ServicePointManager]::SecurityProtocol -bor 3072; iex ((New-Object System.Net.WebClient).DownloadString('https://community.chocolatey.org/install.ps1'))";
    let status = run_script_rs::run_script(&script, false)?;
    anyhow::ensure!(status.success(), "Command failed to install choco");
    tracing::info!("{status:?}");
    Ok(())
}

#[cfg(target_os = "macos")]
fn install_default_manager() -> anyhow::Result<()> {
    println!("Installing homebrew on this system...");
    let script = "curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh";
    let status = run_script_rs::run_script(script, false)?;
    anyhow::ensure!(status.success(), "Command failed to install homebrew");
    tracing::info!("{status:?}");
    Ok(())
}

#[cfg(target_os = "linux")]
fn install_default_manager() {
    println!("This command does nothing on linux.");
}
