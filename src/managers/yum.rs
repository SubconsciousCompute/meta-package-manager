use std::{fmt::Display, process::Command};

use crate::{
    managers::DandifiedYUM, AvailablePackageManager, Cmd, Package, PackageManager,
    PackageManagerCommands, PkgFormat,
};

/// Wrapper for Yellowdog Updater Modified (YUM) package manager.
///
/// [Chapter 14. YUM (Yellowdog Updater Modified) Red Hat Enterprise Linux 5 | Red Hat Customer Portal](https://access.redhat.com/documentation/en-us/red_hat_enterprise_linux/5/html/deployment_guide/c1-yum)
///
/// Note: The current YUM implementation uses [``DandifiedYUM``]'s
/// implementation under the hood, which is why this struct is required to be
/// constructed by calling [``YellowdogUpdaterModified::default()``].
#[derive(Debug)]
pub struct YellowdogUpdaterModified(DandifiedYUM);

impl Default for YellowdogUpdaterModified {
    fn default() -> Self {
        Self(DandifiedYUM)
    }
}

impl Display for YellowdogUpdaterModified {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Yellowdog Updater Modified (YUM)")
    }
}

impl PackageManager for YellowdogUpdaterModified {
    fn pkg_delimiter(&self) -> char {
        self.0.pkg_delimiter()
    }

    fn pkg_manager_name(&self) -> String {
        AvailablePackageManager::Yum.to_string().to_lowercase()
    }

    fn supported_pkg_formats(&self) -> Vec<PkgFormat> {
        vec![PkgFormat::Rpm]
    }

    fn parse_pkg<'a>(&self, line: &str) -> Option<crate::Package> {
        self.0.parse_pkg(line)
    }
    fn add_repo(&self, repo: &Vec<String>) -> anyhow::Result<()> {
        self.0.add_repo(repo)
    }
}

impl PackageManagerCommands for YellowdogUpdaterModified {
    fn cmd(&self) -> Command {
        Command::new("yum")
    }
    fn get_cmds(&self, cmd: crate::Cmd, pkg: Option<&Package>) -> Vec<String> {
        self.0.get_cmds(cmd, pkg)
    }
    fn get_flags(&self, cmd: Cmd) -> Vec<String> {
        self.0.get_flags(cmd)
    }
}
