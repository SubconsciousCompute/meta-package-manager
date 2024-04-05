pub mod parser;

#[macro_use]
pub mod print;

use parser::{Cli, MpmCommands};

use crate::common::{Package, PackageManager};

/// Function that handles the parsed CLI arguments in one place
pub fn execute(args: Cli) -> anyhow::Result<()> {
    let mpm = if let Some(manager) = args.manager {
        crate::MetaPackageManager::try_new(manager)?
    } else {
        crate::MetaPackageManager::default()?
    };

    match args.command {
        MpmCommands::Managers => print::print_managers(),
        MpmCommands::Search { string } => {
            let pkgs = mpm.search(&string);
            println!("{pkgs:#?}");
        }
        MpmCommands::List => {
            let pkgs = mpm.list_installed();
            println!("{pkgs:#?}");
        }
        MpmCommands::Install { packages } => {
            for pkg in packages {
                let s = mpm.install(Package::from_str(&pkg));
                anyhow::ensure!(s.success(), "Failed to install {pkg}");
            }
        }
        MpmCommands::Uninstall { packages } => {
            for pkg in packages {
                let s = mpm.uninstall(Package::from_str(&pkg));
                anyhow::ensure!(s.success(), "Failed to uninstall pacakge {pkg}");
            }
        }

        MpmCommands::Update { packages, all } => {
            if all {
                mpm.update_all();
            } else {
                for pkg in packages {
                    let s = mpm.update(Package::from_str(&pkg));
                    anyhow::ensure!(s.success(), "Failed to update pacakge {pkg}");
                }
            }
        }
        MpmCommands::Repo { repo } => {
            mpm.add_repo(&repo)?;
        }
        MpmCommands::Sync => {
            let s = mpm.sync();
            anyhow::ensure!(s.success(), "Failed to sync repositories");
        }
    };

    Ok(())
}
