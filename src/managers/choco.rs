use std::{fmt::Display, process::Command};

use crate::{common::Package, Cmd, PackageManager, PackageManagerCommands, PkgFormat};

/// Wrapper for the Chocolatey package manager for windows
///
/// [Chocolatey Software | Chocolatey - The package manager for Windows](https://chocolatey.org/)
#[derive(Debug, Default)]
pub struct Chocolatey;

impl PackageManager for Chocolatey {
    fn pkg_delimiter(&self) -> char {
        '|'
    }

    /// Reformat for chocolatey.
    fn reformat_for_command(&self, pkg: &mut Package) -> String {
        if let Some(v) = pkg.version() {
            format!("{} --version {}", pkg.name(), v)
        } else {
            pkg.name().to_string()
        }
    }

    fn supported_pkg_formats(&self) -> Vec<PkgFormat> {
        vec![PkgFormat::Msi, PkgFormat::Exe]
    }
}

impl PackageManagerCommands for Chocolatey {
    fn cmd(&self) -> Command {
        Command::new("choco")
    }
    fn get_cmds(&self, cmd: Cmd, _pkg: Option<&Package>) -> Vec<String> {
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
	    Cmd::Outdated => vec!["outdated", "--limit-output"],
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
    fn test_choco_pkg_fmt() {
        assert_eq!(
            Chocolatey.reformat_for_command(&mut "package".into()),
            "package".to_string()
        );
        assert_eq!(
            &Chocolatey.reformat_for_command(&mut "package@0.1.0".into()),
            "package --version 0.1.0"
        );
    }

    #[cfg(windows)]
    #[test]
    fn test_chocolatey() {
        let choco = Chocolatey;
        let pkg = "tac";
        // sync
        assert!(choco.sync().success());
        // search
        assert!(choco.search(pkg).iter().any(|p| p.name() == pkg));
        // install
        assert!(choco.install(pkg).success());
        // list
        assert!(choco.list_installed().iter().any(|p| p.name() == pkg));
        // update
        assert!(choco.update(pkg).success());
        // uninstall
        assert!(choco.uninstall(pkg).success());
        // TODO: Test AddRepo
    }
}
