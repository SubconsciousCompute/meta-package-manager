pub mod parser;

#[macro_use]
pub mod print;

use parser::Cli;
use parser::MpmCommands;

/// Function that handles the parsed CLI arguments in one place
pub fn execute(args: Cli) -> anyhow::Result<()> {
    let mpm = if let Some(manager) = args.manager {
        crate::MetaPackageManager::try_new(manager)?
    } else {
        crate::MetaPackageManager::default()?
    };

    match args.command {
        MpmCommands::Managers => print::print_managers(),
        MpmCommands::Search { string } => mpm.search(&string)?,
        MpmCommands::List => mpm.list()?,
        MpmCommands::Install { packages } => mpm.install(packages)?,
        MpmCommands::Uninstall { packages } => mpm.uninstall(packages)?,
        MpmCommands::Update { packages, all } => {
            if all {
                mpm.update_all()?
            } else {
                mpm.update(packages)?;
            }
        }
        MpmCommands::Repo { repo } => mpm.add_repo(&repo)?,
        MpmCommands::Sync => mpm.sync()?,
    };

    Ok(())
}
