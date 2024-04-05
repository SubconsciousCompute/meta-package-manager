use std::{fmt::Display, process::Command};

use crate::{common::Package, Cmd, Commands, PackageManager};

/// Wrapper for the Chocolatey package manager for windows
///
/// [Chocolatey Software | Chocolatey - The package manager for Windows](https://chocolatey.org/)
#[derive(Debug, Default)]
pub struct Chocolatey;

impl PackageManager for Chocolatey {
    fn pkg_delimiter(&self) -> char {
        '|'
    }
    fn pkg_format(&self, pkg: &Package) -> String {
        if let Some(v) = pkg.version() {
            format!("{} --version {}", pkg.name(), v)
        } else {
            pkg.name().into()
        }
    }
}

impl Commands for Chocolatey {
    fn cmd(&self) -> Command {
        Command::new("choco")
    }
    fn get_cmds(&self, cmd: Cmd) -> Vec<String> {
        match cmd {
            Cmd::Install => vec!["install"],
            Cmd::Uninstall => vec!["uninstall"],
            Cmd::Update => vec!["upgrade"],
            Cmd::UpdateAll => vec!["upgrade", "all"],
            Cmd::List => vec!["list"],
            // Since chocolatey does not have an analogue for sync command
            // updating chocolatey was chosen as an alternative
            Cmd::Sync => vec!["upgrade", "chocolatey"],
            Cmd::AddRepo => vec!["source", "add"],
            Cmd::Search => vec!["search"],
        }
        .iter()
        .map(|x| x.to_string())
        .collect()
    }
    fn get_flags(&self, cmd: Cmd) -> Vec<String> {
        match cmd {
            Cmd::List | Cmd::Search => vec!["--limit-output"],
            Cmd::Install | Cmd::Update | Cmd::UpdateAll => vec!["--yes"],
            _ => vec![],
        }
        .iter()
        .map(|x| x.to_string())
        .collect()
    }
}

impl Display for Chocolatey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Chocolatey")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn choco_pkg_fmt() {
        let pkg = Package::from("package");
        assert_eq!(Chocolatey.pkg_format(&pkg), Cow::from("package"));
        let pkg = pkg.with_version("0.1.0");
        assert_eq!(
            Chocolatey.pkg_format(&pkg),
            Cow::from("package --version 0.1.0")
        );
    }
}
