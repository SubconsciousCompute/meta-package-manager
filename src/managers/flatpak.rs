use std::{fmt::Display, process::Command};

use crate::{
    AvailablePackageManager, Cmd, Package, PackageManager, PackageManagerCommands, PkgFormat,
};

/// Wrapper for flatpak, which provides sandboxed, cross-distribution,
/// and dependency-free application packaging for linux.
#[derive(Debug, Default)]
pub struct Flatpak;

impl PackageManager for Flatpak {
    fn pkg_delimiter(&self) -> char {
        '-'
    }

    fn pkg_manager_name(&self) -> String {
        AvailablePackageManager::Flatpak.to_string().to_lowercase()
    }

    fn supported_pkg_formats(&self) -> Vec<PkgFormat> {
        vec![PkgFormat::Flatpak]
    }

    fn parse_pkg<'a>(&self, line: &str) -> Option<Package> {
        let mut row = line.split('\t');
        let count = row.clone().count();

        match count {
            4 => {
                let name = row.nth(1)?;
                let ver = row.nth(0)?;
                Some(Package::new(name, self.pkg_manager_name(), Some(ver)))
            }
            5 => {
                let name = row.nth(1)?;
                let ver = row.nth(0)?;
                Some(Package::new(name, self.pkg_manager_name(), Some(ver)))
            }
            6 => {
                let name = row.nth(2)?;
                let ver = row.nth(0)?;
                Some(Package::new(name, self.pkg_manager_name(), Some(ver)))
            }
            _ => None,
        }
    }
}

impl Display for Flatpak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Flatpak")
    }
}

impl PackageManagerCommands for Flatpak {
    fn cmd(&self) -> Command {
        Command::new("flatpak")
    }

    fn get_cmds(&self, cmd: Cmd, _pkg: Option<&Package>) -> Vec<String> {
        match cmd {
            Cmd::Install => vec!["install"],
            Cmd::Uninstall => vec!["uninstall"],
            Cmd::Update => vec!["update"],
            Cmd::UpdateAll => vec!["update"],
            Cmd::List => vec!["list"],
            Cmd::Sync => vec![],
            Cmd::AddRepo => vec!["remote-add"],
            Cmd::Search => vec!["search"],
            Cmd::Outdated => vec!["remote-ls", "--updates", "flathub"],
        }
        .iter()
        .map(|x| x.to_string())
        .collect()
    }

    fn get_flags(&self, cmd: Cmd) -> Vec<String> {
        match cmd {
            Cmd::Install | Cmd::Uninstall | Cmd::Update | Cmd::UpdateAll => vec!["-y"],
            Cmd::AddRepo => vec!["--if-not-exists"],
            _ => vec![],
        }
        .iter()
        .map(|x| x.to_string())
        .collect()
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_imports)]
    use std::str::FromStr;

    use super::Flatpak;
    use crate::{Package, PackageManager, PackageManagerCommands};

    #[test]
    fn test_parse_pkg() {
        let input = r#"
Flatpak Developer Demo	Flatpak Developer Demo	org.flatpak.qtdemo	1.1.3	stable	flathub
Blender	Free and open source 3D creation suite	org.blender.Blender	4.1	stable	fedora,flathub
Inkscape	Vector Graphics Editor	org.inkscape.Inkscape	1.3.2	stable	fedora,flathub"#;
        let flatpak = Flatpak;
        let mut iter = input.lines().filter_map(|l| flatpak.parse_pkg(l));
        assert_eq!(
            iter.next(),
            Package::from_str("flatpak@org.flatpak.qtdemo@1.1.3").ok()
        );
        assert_eq!(
            iter.next(),
            Package::from_str("flatpak@org.blender.Blender@4.1").ok()
        );
        assert_eq!(
            iter.next(),
            Package::from_str("flatpak@org.inkscape.Inkscape@1.3.2").ok()
        );
    }

    // Requires elevated privilages to work
    #[cfg(target_os = "linux")]
    #[tracing_test::traced_test]
    #[test]
    fn test_flatpak() {
        let flatpak = crate::managers::Flatpak;
        if !flatpak.is_available() {
            println!("flatpak is not available");
            return;
        }

        let pkg = "org.flatpak.qtdemo";

        // search
        let found_pkgs = flatpak.search(pkg);
        tracing::info!("Found packages: {found_pkgs:#?}");
        tracing::info!(
            "Found packages: {:#?}",
            found_pkgs
                .iter()
                .map(|p| p.cli_display(flatpak.pkg_delimiter()))
                .collect::<Vec<_>>()
        );
        assert!(found_pkgs.iter().any(|p| p.name() == "org.flatpak.qtdemo"));

        // install
        assert!(flatpak.install(pkg).success());
        // list
        assert!(flatpak
            .list_installed()
            .iter()
            .any(|p| p.name() == "org.flatpak.qtdemo"));
        // update
        assert!(flatpak.update(pkg).success());
        // uninstall
        assert!(flatpak.uninstall(pkg).success());
    }
}
