use anyhow::{bail, Context, Result};
use manager::Manager;
use strum::IntoEnumIterator;

mod manager;

pub mod parser;
use parser::{Cli, Commands};

#[macro_use]
pub mod print;

use crate::{verify::DynVerified, Operation};

/// Primary interface to executing the CLI commands
/// "PkgManagerHandler" because it handles Package Managers, and it's funny
pub struct PkgManagerHandler(Option<Manager>);

impl Default for PkgManagerHandler {
    fn default() -> Self {
        Self::new(None)
    }
}

impl PkgManagerHandler {
    /// Initialize with an optional package manager
    pub fn new(man: Option<Manager>) -> Self {
        Self(man)
    }

    /// Tries to initialize the package manager PkgManagerHandler was
    /// initialized with if its present, or else it tries to get the default
    /// one.
    fn get_man(&self) -> Result<DynVerified> {
        let man = if let Some(m) = &self.0 {
            let userm = m
                .init()
                .context("requested package manager is unavailable")?;
            notify!("running command(s) through {userm}");
            userm
        } else {
            let defm = Self::get_default()?;
            notify!("defaulting to {defm}");
            defm
        };
        Ok(man)
    }

    /// First enum variant is given the highest priority, second, the second
    /// highest, and so on.
    pub fn get_default() -> Result<DynVerified> {
        Manager::iter()
            .find_map(|m| m.init())
            .context("no supported package manager found")
    }

    /// Wrapper for [``Self::execute_op``]
    pub fn install(&self, pkgs: Vec<String>) -> Result<()> {
        self.execute_op(pkgs, Operation::Install)
    }

    /// Wrapper for [``Self::execute_op``]
    pub fn uninstall(&self, pkgs: Vec<String>) -> Result<()> {
        self.execute_op(pkgs, Operation::Uninstall)
    }

    /// Wrapper for [``Self::execute_op``]
    pub fn update(&self, pkgs: Vec<String>) -> Result<()> {
        self.execute_op(pkgs, Operation::Update)
    }

    /// Execute the update_all operation on the package manager
    pub fn update_all(&self) -> Result<()> {
        let man = self.get_man()?;
        let status = man.update_all();
        if !status.success() {
            bail!("failed to update all packages using {man} with {status}");
        }
        Ok(())
    }

    /// Handles three different types of [``Operation``]s on packages: Install,
    /// Uninstall and Update
    fn execute_op(&self, raw_pkgs: Vec<String>, op: Operation) -> Result<()> {
        let pkgs: Vec<_> = raw_pkgs.iter().map(|p| parser::pkg_parse(p)).collect();
        let man = self.get_man()?;
        let status = man.exec_op(&pkgs, op);
        if !status.success() {
            bail!(
                "failed to execute {:?} operation using {man} with {status}",
                op,
            );
        }
        Ok(())
    }

    /// Does the same as the [``PackageManager::add_repo``] fn
    pub fn add_repo(&self, repo: &str) -> Result<()> {
        let man = self.get_man()?;
        if let Err(err) = man.add_repo(repo) {
            bail!("{err}")
        }
        Ok(())
    }

    /// Does the same as the [``PackageManager::sync``] fn
    pub fn sync(&self) -> Result<()> {
        let man = self.get_man()?;
        let status = man.sync();
        if !status.success() {
            bail!("failed to sync {man} due to {status}")
        }
        Ok(())
    }

    /// Does the same as the [``PackageManager::list``] fn
    pub fn list(&self) -> Result<()> {
        let man = self.get_man()?;
        let pkgs = man.list_installed();
        notify!("{} packages found", pkgs.len());
        if !pkgs.is_empty() {
            print::print_packages(pkgs);
        }
        Ok(())
    }

    /// Does the same as the [``PackageManager::search``] fn
    pub fn search(&self, query: &str) -> Result<()> {
        tracing::debug!("Searching for {query}");
        let man = self.get_man()?;
        let pkgs = man.search(query);
        if pkgs.is_empty() {
            bail!("no packages found that match the query: {query}")
        }
        notify!("{} packages found for query {query}", pkgs.len());
        print::print_packages(pkgs);
        Ok(())
    }
}

/// Function that handles the parsed CLI arguments in one place
pub fn execute(args: Cli) -> Result<()> {
    let handler = PkgManagerHandler::new(args.manager);
    match args.command {
        Commands::Managers => print::print_managers(),
        Commands::Search { string } => handler.search(&string)?,
        Commands::List => handler.list()?,
        Commands::Install { packages } => handler.install(packages)?,
        Commands::Uninstall { packages } => handler.uninstall(packages)?,
        Commands::Update { packages, all } => {
            if all {
                handler.update_all()?
            } else {
                handler.update(packages)?;
            }
        }
        Commands::Repo { repo } => handler.add_repo(&repo)?,
        Commands::Sync => handler.sync()?,
    };

    Ok(())
}
