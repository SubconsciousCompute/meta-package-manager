use std::{fmt::Display, process::Command};

use crate::{
    AvailablePackageManager, Cmd, Package, PackageManager, PackageManagerCommands, PkgFormat,
};

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

    fn pkg_manager_name(&self) -> String {
        AvailablePackageManager::Dnf.to_string().to_lowercase()
    }

    fn supported_pkg_formats(&self) -> Vec<PkgFormat> {
        vec![PkgFormat::Rpm]
    }

    fn parse_pkg<'a>(&self, line: &str) -> Option<Package> {
        if line.contains('@') || line.split_whitespace().count() == 3 {
            let mut splt = line.split_whitespace();
            let name = splt.next()?;
            let ver = splt.next()?;
            return Some(Package::new(
                name.trim(),
                self.pkg_manager_name(),
                Some(ver.trim()),
            ));
        }
        if line.contains('^') {
            let (name, ver) = line.split_once('^')?;
            return Some(Package::new(
                name.trim(),
                self.pkg_manager_name(),
                Some(ver.trim()),
            ));
        }
        if !line.contains("====") {
            Some(Package::new(
                line.split_once(':')?.0.trim(),
                self.pkg_manager_name(),
                None,
            ))
        } else {
            None
        }
    }

    fn add_repo(&self, repo: &Vec<String>) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.install(
                Package::new("dnf-command(config-manager)", self.pkg_manager_name(), None),
                false
            )
            .success(),
            "failed to install config-manager plugin"
        );

        let s = self.exec_cmds_status(&self.consolidated(Cmd::AddRepo, None, repo), None);
        anyhow::ensure!(s.success(), "failed to add repo");
        Ok(())
    }
}

impl Display for DandifiedYUM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Dandified YUM (DNF)")
    }
}

impl PackageManagerCommands for DandifiedYUM {
    fn cmd(&self) -> Command {
        Command::new("dnf")
    }

    fn get_cmds(&self, cmd: Cmd, _pkg: Option<&Package>) -> Vec<String> {
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
            Cmd::Outdated => vec!["repoquery", "--upgrades", "--qf", "%{name}^%{version}\n"],
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
    use std::str::FromStr;

    use super::DandifiedYUM;
    use crate::{Package, PackageManager};

    #[test]
    fn test_parse_pkg() {
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
            Package::from_str("dnf@sudo.x86_64@1.9.13-2.p2.fc38").ok()
        );
        assert_eq!(
            iter.next(),
            Package::from_str("dnf@systemd-libs.x86_64@253.10-1.fc38").ok()
        );

        assert_eq!(iter.next(), Package::from_str("dnf@hello.x86_64").ok());
        assert_eq!(
            iter.next(),
            Package::from_str("dnf@rubygem-mixlib-shellout-doc.noarch").ok()
        );
    }

    // Requires elevated privilages to work
    #[cfg(target_os = "linux")]
    #[test]
    fn test_dnf() {
        dnf_yum_cases(crate::managers::DandifiedYUM)
    }
    #[cfg(target_os = "linux")]
    fn dnf_yum_cases(man: impl crate::PackageManager) {
        if !man.is_available() {
            println!("Yum is not available");
            return;
        }
        let pkg = "hello";
        // sync
        assert!(man.sync().success());
        // search
        let found_pkgs = man.search(pkg);
        tracing::info!("Found packages: {found_pkgs:#?}");
        tracing::info!(
            "Found packages: {:#?}",
            found_pkgs
                .iter()
                .map(|p| p.cli_display(man.pkg_delimiter()))
                .collect::<Vec<_>>()
        );
        assert!(found_pkgs.iter().any(|p| p.name() == "hello.x86_64"));

        // install
        assert!(man.install(pkg, false).success());
        // list
        assert!(man
            .list_installed()
            .iter()
            .any(|p| p.name() == "hello.x86_64"));
        // update
        assert!(man.update(pkg, false).success());
        // uninstall
        assert!(man.uninstall(pkg, false).success());
        // TODO: Test AddRepo
    }
}
