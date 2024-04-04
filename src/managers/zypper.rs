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
                packages.push(Package {
                    name: p
                        .attributes
                        .get("name")
                        .expect("must have name")
                        .to_string()
                        .into(),
                    version: None,
                })
            }
        }
        packages
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
            Cmd::List => &["--xmlout", "search"],
            Cmd::Sync => &["refresh"],
            Cmd::AddRepo => &["addrepo"],
            Cmd::Search => &["--xmlout", "search"],
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
