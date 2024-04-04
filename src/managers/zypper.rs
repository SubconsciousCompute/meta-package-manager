//! Zypper package manager

use crate::{Cmd, Commands, Package, PackageManager, RepoError};
use std::{fmt::Display, process::Command};

/// Wrapper for Zypper package manager. Some openSUSE might support dnf as well.
#[derive(Debug)]
pub struct Zypper;

impl PackageManager for Zypper {
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

impl Display for Zypper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("zypper")
    }
}

impl Commands for Zypper {
    /// return a primary command.
    fn cmd(&self) -> Command {
        Command::new("zypper")
    }

    fn get_cmds(&self, cmd: Cmd) -> &'static [&'static str] {
        match cmd {
            Cmd::Install => &["install"],
            Cmd::Uninstall => &["remove"],
            Cmd::Update => &["update"],
            Cmd::UpdateAll => &["dist-upgrade"],
            Cmd::List => &["search"],
            Cmd::Sync => &["refresh"],
            Cmd::AddRepo => &["addrepo"],
            Cmd::Search => &["search"],
        }
    }

    fn get_flags(&self, cmd: Cmd) -> &'static [&'static str] {
        match cmd {
            Cmd::Install | Cmd::Uninstall | Cmd::Update | Cmd::UpdateAll => &["-n"],
            Cmd::List => &["-i"],
            Cmd::Search => &["--no-refresh", "-q"],
            Cmd::AddRepo => &["-f"],
            _ => &["-n"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[test]
    #[traced_test]
    fn test_generate_cmd_zypper() {
        let _zypper = Zypper;
    }
}
