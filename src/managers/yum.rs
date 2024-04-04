use std::{fmt::Display, process::Command};

use crate::{managers::DandifiedYUM, Cmd, Commands, PackageManager};

/// Wrapper for Yellowdog Updater Modified (YUM) package manager.
///
/// [Chapter 14. YUM (Yellowdog Updater Modified) Red Hat Enterprise Linux 5 | Red Hat Customer Portal](https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/5/html/deployment_guide/c1-yum)
///
/// Note: The current YUM implementation uses [``DandifiedYUM``]'s
/// implementation under the hood, which is why this struct is required to be
/// constructed by calling [``YellowdogUpdaterModified::default()``].
#[derive(Debug)]
pub struct YellowdogUpdaterModified {
    dnf: DandifiedYUM,
}

impl Default for YellowdogUpdaterModified {
    fn default() -> Self {
        Self { dnf: DandifiedYUM }
    }
}

impl Display for YellowdogUpdaterModified {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Yellowdog Updater Modified (YUM)")
    }
}

impl PackageManager for YellowdogUpdaterModified {
    fn pkg_delimiter(&self) -> char {
        self.dnf.pkg_delimiter()
    }
    fn parse_pkg<'a>(&self, line: &str) -> Option<crate::Package<'a>> {
        self.dnf.parse_pkg(line)
    }
    fn add_repo(&self, repo: &str) -> Result<(), crate::RepoError> {
        self.dnf.add_repo(repo)
    }
}

impl Commands for YellowdogUpdaterModified {
    fn cmd(&self) -> Command {
        Command::new("yum")
    }
    fn get_cmds(&self, cmd: crate::Cmd) -> &'static [&'static str] {
        self.dnf.get_cmds(cmd)
    }
    fn get_flags(&self, cmd: Cmd) -> &'static [&'static str] {
        self.dnf.get_flags(cmd)
    }
}
