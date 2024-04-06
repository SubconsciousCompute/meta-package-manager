use std::{fmt::Display, fs, io::Write, process::Command};

use crate::{common::Package, Cmd, PackageManager, PackageManagerCommands};

/// Wrapper for Advanced Pacakge Tool (APT), the default package management
/// user-facing utilities in Debian and Debian-based distributions.
///
/// [Apt - Debian Wiki](https://wiki.debian.org/Apt)
/// # Idiosyncracies
/// [``AdvancedPackageTool::list_installed``] and
/// [``AdvancedPackageTool::search``] internally depend on "apt" command
/// while the rest depend on "apt-get" command.
///
/// Another notable point is that the [``AdvancedPackageTool::add_repo``]
/// implementation doesn't execute commands, but it writes to
/// "/etc/apt/sources.list".
#[derive(Debug, Default)]
pub struct AdvancedPackageTool;

impl AdvancedPackageTool {
    const SOURCES: &'static str = "/etc/apt/sources.list";

    fn alt_cmd<S: AsRef<str>>(&self, cmds: &[S]) -> Command {
        if matches!(
            cmds.first().map(AsRef::as_ref),
            Some("list") | Some("search")
        ) {
            Command::new("apt")
        } else {
            self.cmd()
        }
    }
}

impl PackageManager for AdvancedPackageTool {
    fn pkg_delimiter(&self) -> char {
        '='
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

    fn add_repo(&self, repo: &str) -> anyhow::Result<()> {
        let mut sources = fs::File::options().append(true).open(Self::SOURCES)?;
        sources.write_fmt(format_args!("\n{}", repo))?;
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
        Command::new("apt-get")
    }
    fn get_cmds(&self, cmd: Cmd) -> Vec<String> {
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

    fn exec_cmds(&self, cmds: &[String]) -> std::process::Output {
        self.alt_cmd(cmds).args(cmds).output().unwrap()
    }

    fn exec_cmds_status<S: AsRef<str>>(&self, cmds: &[S]) -> std::process::ExitStatus {
        self.alt_cmd(cmds)
            .args(cmds.iter().map(AsRef::as_ref))
            .status()
            .unwrap()
    }

    fn exec_cmds_spawn(&self, cmds: &[String]) -> std::process::Child {
        self.alt_cmd(cmds).args(cmds).spawn().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::AdvancedPackageTool;
    use crate::{Cmd, Package, PackageManager, PackageManagerCommands};

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

    #[test]
    fn alt_cmd() {
        let apt = AdvancedPackageTool;
        let alt = "apt";
        let reg = "apt-get";
        let cmds = &[
            Cmd::Install,
            Cmd::Uninstall,
            Cmd::Update,
            Cmd::UpdateAll,
            Cmd::List,
            Cmd::Sync,
            Cmd::AddRepo,
            Cmd::Search,
        ];

        for cmd in cmds.iter() {
            let should_match = match cmd {
                Cmd::Search | Cmd::List => alt,
                _ => reg,
            };
            assert_eq!(apt.alt_cmd(&apt.get_cmds(*cmd)).get_program(), should_match);
        }
    }
}
