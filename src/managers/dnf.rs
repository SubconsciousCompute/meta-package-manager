use crate::{Cmd, Commands, Package, PackageManager, RepoError};
use std::{fmt::Display, process::Command};

/// Wrapper for DandifiedYUM or DNF, the next upcoming major version of YUM
///
/// [DNF, the next-generation replacement for YUM — dnf latest documentation](https://dnf.readthedocs.io/en/latest/)
/// # Idiosyncracies
/// The [``DandifiedYUM::add_repo``] method also installs `config-manager`
/// plugin for DNF before attempting to add a repo.
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
