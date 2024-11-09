use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
    str::FromStr,
};

use clap::{Parser, Subcommand, ValueEnum};
use strum::IntoEnumIterator;

use crate::{
    AvailablePackageManager, MetaPackageManager, Package, PackageManager, PackageManagerCommands,
};

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
    List {
        #[arg(long, short)]
        all: bool,

        #[arg(short, long, value_enum)]
        output: Option<FileFormat>,
    },

    #[command(
        about = "Install the given package(s)",
        long_about = "Install the given package(s).\nIf a specific version of the package is desired, it can be specified using the format <package_name>@<version>.\nNote: version information is optional."
    )]
    Install {
        #[arg(required_unless_present = "input_file")]
        packages: Vec<String>,

        #[arg(short, long, required_unless_present = "packages")]
        input_file: Option<PathBuf>,
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
    Repo { repo: Vec<String> },

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

    #[command(about = "List all of the packages that can be updated")]
    Outdated,
}

#[derive(Clone, ValueEnum)]
pub enum FileFormat {
    Toml,
    Json,
    None,
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
        MpmPackageManagerCommands::Install { .. }
            | MpmPackageManagerCommands::Uninstall { .. }
            | MpmPackageManagerCommands::Update { .. }
            | MpmPackageManagerCommands::Repo { .. }
            | MpmPackageManagerCommands::Sync
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
        MpmPackageManagerCommands::List { all, output } => {
            let pkgs;

            if all {
                pkgs = list_all_installed();
            } else {
                pkgs = mpm.list_installed();
            }

            match output {
                Some(FileFormat::Toml) => pkgs_to_format(&pkgs, FileFormat::Toml)?,
                Some(FileFormat::Json) => pkgs_to_format(&pkgs, FileFormat::Json)?,
                Some(FileFormat::None) => (),
                _ => print_pkgs(&pkgs, args.json)?,
            };
        }
        MpmPackageManagerCommands::Install {
            packages,
            input_file,
        } => {
            if input_file.is_some() {
                let input = input_file.unwrap();
                let file_type = get_file_type(&input);
                install_from_file(&input, file_type)?;
                return Ok(());
            }

            for pkg in packages {
                let pkg_path = PathBuf::from(&pkg);
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
        MpmPackageManagerCommands::Outdated => {
            let pkgs = mpm.list_outdated();
            print_pkgs(&pkgs, args.json)?;
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

/// Convert Package to a JSON or TOML format
fn pkgs_to_format(packages: &[Package], format: FileFormat) -> anyhow::Result<()> {
    let mut grouped: BTreeMap<String, HashMap<String, String>> = BTreeMap::new();

    for package in packages {
        if let Some(version) = &package.version() {
            grouped
                .entry(package.package_manager().to_string())
                .or_default()
                .insert(package.name().to_string(), version.to_string());
        }
    }

    let output = match format {
        FileFormat::Toml => toml::to_string(&grouped)?,
        FileFormat::Json => serde_json::to_string_pretty(&grouped)?,
        FileFormat::None => todo!(),
    };

    println!("{}", output);

    Ok(())
}

/// List all of the installed packages from all of the available package
/// managers
fn list_all_installed() -> Vec<Package> {
    let mut all_packages = HashSet::new();

    for pm in AvailablePackageManager::iter() {
        let mpm = MetaPackageManager::new(pm.clone());
        if mpm.is_available() {
            let packages = mpm.list_installed();
            all_packages.extend(packages);
        }
    }

    all_packages.into_iter().collect()
}

/// Get the input file format
fn get_file_type(file: &PathBuf) -> FileFormat {
    match file.extension().and_then(|ext| ext.to_str()) {
        Some("json") => FileFormat::Json,
        Some("toml") => FileFormat::Toml,
        _ => FileFormat::None,
    }
}

/// Install a list of packages from a given input file
fn install_from_file(input_file: &PathBuf, file_type: FileFormat) -> anyhow::Result<()> {
    type PackageMap = HashMap<String, HashMap<String, String>>;

    let file_contents = std::fs::read_to_string(input_file)?;

    let parsed: PackageMap = match file_type {
        FileFormat::Json => serde_json::from_str(&file_contents)?,
        FileFormat::Toml => toml::from_str(&file_contents)?,
        FileFormat::None => todo!(),
    };

    for (package_manager, packages) in parsed {
        let pm = match package_manager.as_str() {
            "apt" => AvailablePackageManager::Apt,
            "brew" => AvailablePackageManager::Brew,
            "choco" => AvailablePackageManager::Choco,
            "dnf" => AvailablePackageManager::Dnf,
            "flatpak" => AvailablePackageManager::Flatpak,
            "yum" => AvailablePackageManager::Yum,
            "zypper" => AvailablePackageManager::Zypper,
            _ => todo!(),
        };

        let mpm = MetaPackageManager::new(pm.clone());

        for (name, _version) in packages {
            if mpm.is_available() {
                let s = mpm.install(name.as_str());
                anyhow::ensure!(s.success(), "Failed to install {name}");
            }
        }
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
    let status = run_script_rs::run_script(script, false)?;
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
fn install_default_manager() -> anyhow::Result<()> {
    println!("This command does nothing on linux.");
    Ok(())
}

#[cfg(target_os = "android")]
fn install_default_manager() -> anyhow::Result<()> {
    println!("This command does nothing on android.");
    Ok(())
}
