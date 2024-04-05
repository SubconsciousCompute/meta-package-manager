use std::{fmt::Display, process::Command};

use crate::{Cmd, Commands, Package, PackageManager};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Wrapper for DandifiedYUM or DNF, the next upcoming major version of YUM
///
/// [DNF, the next-generation replacement for YUM — dnf latest documentation](https://dnf.readthedocs.io/en/latest/)
/// # Idiosyncracies
/// The [``DandifiedYUM::add_repo``] method also installs `config-manager`
/// plugin for DNF before attempting to add a repo.
#[derive(Debug, Default)]
pub struct DandifiedYUM;

impl PackageManager for DandifiedYUM {
    fn pkg_delimiter(&self) -> char {
        '-'
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

    fn add_repo(&self, repo: &str) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.install(Package::new("dnf-command(config-manager)", None))
                .success(),
            "failed to install config-manager plugin"
        );

        let s = self.exec_cmds_status(&self.consolidated(Cmd::AddRepo, &[repo]));
        anyhow::ensure!(s.success(), "failed to add repo");
        Ok(())
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

    fn get_cmds(&self, cmd: Cmd) -> Vec<String> {
        match cmd {
            Cmd::Install => vec!["install"],
            Cmd::Uninstall => vec!["remove"],
            Cmd::Update => vec!["upgrade"],
            Cmd::UpdateAll => vec!["distro-sync"],
            Cmd::List => vec!["list"],
            Cmd::Sync => vec!["makecache"],
            // depends on config-manager plugin (handled in add_repo method)
            Cmd::AddRepo => vec!["config-manager", "--add-repo"], // flag must come before repo
            Cmd::Search => vec!["search"],
        }
        .iter()
        .map(|x| x.to_string())
        .collect()
    }

    fn get_flags(&self, cmd: Cmd) -> Vec<String> {
        match cmd {
            Cmd::Install | Cmd::Uninstall | Cmd::Update | Cmd::UpdateAll => vec!["-y"],
            Cmd::List => vec!["--installed"],
            Cmd::Search => vec!["-q"],
            _ => vec![],
        }
        .iter()
        .map(|x| x.to_string())
        .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::DandifiedYUM;
    use crate::{Package, PackageManager};

    #[test]
    fn parse_pkg() {
        let dnf = DandifiedYUM;
        let input = r#"
sudo.x86_64                                                                                   1.9.13-2.p2.fc38                                                                @koji-override-0
systemd-libs.x86_64                                                                           253.10-1.fc38                                                                   @koji-override-0
================================================================================ Name Exactly Matched: hello =================================================================================
hello.x86_64 : Prints a familiar, friendly greeting
=============================================================================== Name & Summary Matched: hello ================================================================================
rubygem-mixlib-shellout-doc.noarch : Documentation for rubygem-mixlib-shellout"#;

        let mut iter = input.lines().filter_map(|l| dnf.parse_pkg(l));
        assert_eq!(
            iter.next(),
            Some(Package::from("sudo.x86_64").with_version("1.9.13-2.p2.fc38"))
        );
        assert_eq!(
            iter.next(),
            Some(Package::from("systemd-libs.x86_64").with_version("253.10-1.fc38"))
        );
        assert_eq!(iter.next(), Some(Package::from("hello.x86_64")));
        assert_eq!(
            iter.next(),
            Some(Package::from("rubygem-mixlib-shellout-doc.noarch"))
        );
    }
}
