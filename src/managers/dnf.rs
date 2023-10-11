use crate::{Cmd, Commands, Package, PackageManager, RepoError};
use std::{fmt::Display, process::Command};

/// Wrapper for DandifiedYUM or DNF, the default package manager for Fedora
///
/// # Idiosyncracies
/// The [``DandifiedYUM::add_repo``] method also installs `config-manager` plugin for DNF
/// before attempting to add a repo.
#[derive(Debug)]
pub struct DandifiedYUM;

impl PackageManager for DandifiedYUM {
    fn pkg_delimiter(&self) -> char {
        '-'
    }
    fn parse_pkg<'a>(&self, line: &str) -> Option<Package<'a>> {
        if line.contains('@') {
            let mut splt = line.split_whitespace();
            let name = splt.next()?;
            let ver = splt.next()?;
            return Some(Package::from(name.trim().to_owned()).with_version(ver.trim().to_owned()));
        }
        if !line.contains("====") {
            Some(Package::from(line.split_once(':')?.0.trim().to_owned()))
        } else {
            None
        }
    }
    fn add_repo(&self, repo: &str) -> Result<(), RepoError> {
        if !self.install("dnf-command(config-manager)".into()).success() {
            return Err(RepoError::with_msg(
                "failed to install config-manager plugin",
            ));
        }

        self.exec_cmds_status(&self.consolidated(Cmd::AddRepo, &[repo]))
            .success()
            .then_some(())
            .ok_or(RepoError::default())
    }
}

impl Display for DandifiedYUM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Dandified YUM (DNF)")
    }
}

impl Commands for DandifiedYUM {
    fn cmd(&self) -> Command {
        Command::new("dnf")
    }
    fn get_cmds(&self, cmd: Cmd) -> &'static [&'static str] {
        match cmd {
            Cmd::Install => &["install"],
            Cmd::Uninstall => &["remove"],
            Cmd::Update => &["upgrade"],
            Cmd::UpdateAll => &["distro-sync"],
            Cmd::List => &["list"],
            Cmd::Sync => &["makecache"],
            // depends on config-manager plugin (handled in add_repo method)
            Cmd::AddRepo => &["config-manager", "--add-repo"], // flag must come before repo
            Cmd::Search => &["search"],
        }
    }

    fn get_flags(&self, cmd: Cmd) -> &'static [&'static str] {
        match cmd {
            Cmd::Install | Cmd::Uninstall | Cmd::Update | Cmd::UpdateAll => &["-y"],
            Cmd::List => &["--installed"],
            Cmd::Search => &["-q"],
            _ => &[],
        }
    }
}
