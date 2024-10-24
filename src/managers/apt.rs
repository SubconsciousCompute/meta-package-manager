use std::{
    fmt::Display,
    fs,
    io::{BufWriter, Write},
    process::Command,
};

use crate::{common::Package, Cmd, PackageManager, PackageManagerCommands, PkgFormat};

/// Wrapper for Advanced Pacakge Tool (APT), the default package management
/// user-facing utilities in Debian and Debian-based distributions.
///
/// [Apt - Debian Wiki](https://wiki.debian.org/Apt)
/// # Idiosyncracies
/// [``AdvancedPackageTool::list_installed``] and
/// [``AdvancedPackageTool::search``] internally depend on "apt" command
///
/// Another notable point is that the [``AdvancedPackageTool::add_repo``]
/// implementation doesn't execute commands, but it writes to
/// "/etc/apt/sources.list".
#[derive(Debug, Default)]
pub struct AdvancedPackageTool;

impl AdvancedPackageTool {
    const SOURCES: &'static str = "/etc/apt/sources.list";
}

impl PackageManager for AdvancedPackageTool {
    fn pkg_delimiter(&self) -> char {
        '='
    }

    fn supported_pkg_formats(&self) -> Vec<PkgFormat> {
        vec![PkgFormat::Deb]
    }

    fn parse_pkg<'a>(&self, line: &str) -> Option<Package> {
        let (name, info) = line.split_once('/')?;
        if matches!(info.split_whitespace().count(), 3 | 4) {
            let ver = info.split_whitespace().nth(1)?;
            Some(Package::new(name, Some(ver)))
        } else {
            None
        }
    }

    // Apt doesn't support installing from URL.
    fn reformat_for_command(&self, pkg: &mut Package) -> String {
        if let Some(url) = pkg.url() {
            if url.scheme() != "file" {
                tracing::info!(
                    "Apt doesn't support installing directory from URL. Downloading locally..."
                );
                pkg.make_available_on_disk(None, false)
                    .expect("failed to ensure that package exists locally");
            }
        }

        pkg.cli_display(self.pkg_delimiter()).to_string()
    }

    fn add_repo(&self, repo: &Vec<String>) -> anyhow::Result<()> {
        let sources = fs::File::options().append(true).open(Self::SOURCES)?;
        let mut writer = BufWriter::new(sources);

        for line in repo {
            writeln!(writer, "{}", line)?;
        }

        writer.flush()?;

        Ok(())
    }
}

impl Display for AdvancedPackageTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Advanced Package Tool (APT)")
    }
}

impl PackageManagerCommands for AdvancedPackageTool {
    fn cmd(&self) -> Command {
        Command::new("apt")
    }

    fn get_cmds(&self, cmd: Cmd, _pkg: Option<&Package>) -> Vec<String> {
        match cmd {
            Cmd::Install => vec!["install"],
            Cmd::Uninstall => vec!["remove"],
            Cmd::Update => vec!["install"],
            Cmd::UpdateAll => vec!["upgrade"],
            Cmd::List => vec!["list"],
            Cmd::Sync => vec!["update"],
            Cmd::AddRepo => vec![],
            Cmd::Search => vec!["search"],
        }
        .iter()
        .map(|x| x.to_string())
        .collect()
    }

    fn get_flags(&self, cmd: Cmd) -> Vec<String> {
        match cmd {
            Cmd::Install | Cmd::Uninstall | Cmd::UpdateAll => vec!["--yes"],
            Cmd::Update => vec!["--yes", "--only-upgrade"],
            Cmd::List => vec!["--installed"],
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

    use super::AdvancedPackageTool;
    use crate::{Package, PackageManager, PackageManagerCommands};

    #[test]
    fn test_parse_pkg() {
        let input = r#"
hello/stable 2.10-3 amd64
  example package based on GNU hello

iagno/stable 1:3.38.1-2 amd64
  popular Othello game for GNOME

mount/now 2.38.1-5+b1 amd64 [installed,local]
mysql-common/now 5.8+1.1.0 all [installed,local]"#;
        let apt = AdvancedPackageTool;
        let mut iter = input.lines().filter_map(|l| apt.parse_pkg(l));
        assert_eq!(iter.next(), Package::from_str("hello@2.10-3").ok());
        assert_eq!(iter.next(), Package::from_str("iagno@1:3.38.1-2").ok());
        assert_eq!(iter.next(), Package::from_str("mount@2.38.1-5+b1").ok());
        assert_eq!(
            iter.next(),
            Package::from_str("mysql-common@5.8+1.1.0").ok()
        );
    }

    // Requires elevated privilages to work
    #[cfg(target_os = "linux")]
    #[tracing_test::traced_test]
    #[test]
    fn test_apt() {
        let apt = crate::managers::AdvancedPackageTool;
        if !apt.is_available() {
            println!("apt is not available");
            return;
        }

        let pkg = "hello";
        // sync
        assert!(apt.sync().success());

        // search
        let found_pkgs = apt.search(pkg);
        tracing::info!("Found packages: {found_pkgs:#?}");
        tracing::info!(
            "Found packages: {:#?}",
            found_pkgs
                .iter()
                .map(|p| p.cli_display(apt.pkg_delimiter()))
                .collect::<Vec<_>>()
        );
        assert!(found_pkgs.iter().any(|p| p.name() == "hello"));

        // install
        assert!(apt.install(pkg).success());
        // list
        assert!(apt.list_installed().iter().any(|p| p.name() == "hello"));
        // update
        assert!(apt.update(pkg).success());
        // uninstall
        assert!(apt.uninstall(pkg).success());
        // TODO: Test AddRepo
    }
}
