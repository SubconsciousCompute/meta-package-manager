//! Zypper package manager

use std::{fmt::Display, process::Command};

use crate::{Cmd, Commands, Package, PackageManager, RepoError};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Wrapper for Zypper package manager. Some openSUSE might support dnf as well.
#[derive(Debug, Default)]
pub struct Zypper;

impl PackageManager for Zypper {
    fn pkg_delimiter(&self) -> char {
        '-'
    }

    /// Parses output, generally from stdout, to a Vec of Packages.
    ///
    /// The default implementation uses [``PackageManager::parse_pkg``] for
    /// parsing each line into a [`Package`].
    fn parse_output(&self, out: &[u8]) -> Vec<Package> {
        use xmltree::Element;

        let xml = String::from_utf8_lossy(out);
        let root = Element::parse(xml.as_bytes()).expect("invalid xml");
        let list = root
            .get_child("search-result")
            .expect("no search result found")
            .get_child("solvable-list")
            .expect("no solvable-list found");

        let mut packages = vec![];
        for p in &list.children {
            if let Some(p) = p.as_element() {
                packages.push(Package::new(
                    p.attributes.get("name").expect("must have name"),
                    None,
                ));
            }
        }
        packages
    }

    fn parse_pkg<'a>(&self, line: &str) -> Option<Package> {
        if line.contains('@') {
            let mut splt = line.split_whitespace();
            let name = splt.next()?;
            let ver = splt.next()?;
            return Some(Package::new(name.trim(), Some(ver.trim())));
        }
        if !line.contains("====") {
            Some(Package::new(line.split_once(':')?.0.trim(), None))
        } else {
            None
        }
    }

    fn add_repo(&self, repo: &str) -> Result<(), RepoError> {
        if !self
            .install(Package::new("dnf-command(config-manager)", None))
            .success()
        {
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

    fn get_cmds(&self, cmd: Cmd) -> Vec<String> {
        match cmd {
            Cmd::Install => vec!["install"],
            Cmd::Uninstall => vec!["remove"],
            Cmd::Update => vec!["update"],
            Cmd::UpdateAll => vec!["dist-upgrade"],
            Cmd::List => vec!["--xmlout", "search"],
            Cmd::Sync => vec!["refresh"],
            Cmd::AddRepo => vec!["addrepo"],
            Cmd::Search => vec!["--xmlout", "search"],
        }
        .iter()
        .map(|x| x.to_string())
        .collect()
    }

    fn get_flags(&self, cmd: Cmd) -> Vec<String> {
        match cmd {
            Cmd::Install | Cmd::Uninstall | Cmd::Update | Cmd::UpdateAll => vec!["-n"],
            Cmd::List => vec!["-i"],
            Cmd::Search => vec!["--no-refresh", "-q"],
            Cmd::AddRepo => vec!["-f"],
            _ => vec!["-n"],
        }
        .iter()
        .map(|x| x.to_string())
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn test_generate_cmd_zypper() {
        let _zypper = Zypper;
    }
}
