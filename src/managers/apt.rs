use crate::{Cmd, Commands, Package, PackageManager, RepoError};
use std::{fmt::Display, fs, io::Write, process::Command};

/// Wrapper for Advanced Pacakge Tool (APT), the default package management user-facing utilities
/// in Debian and Debian-based distributions.
///
/// [Apt - Debian Wiki](https://wiki.debian.org/Apt)
/// # Idiosyncracies
/// [``AdvancedPackageTool::list_installed``] and [``AdvancedPackageTool::search``] internally depend on "apt" command
/// while the rest depend on "apt-get" command.
///
/// Another notable point is that the [``AdvancedPackageTool::add_repo``] implementation doesn't execute commands, but it writes to "/etc/apt/sources.list".
#[derive(Debug)]
pub struct AdvancedPackageTool;

impl AdvancedPackageTool {
    const SOURCES: &str = "/etc/apt/sources.list";

    fn alt_cmd(cmds: &[&str]) -> Command {
        if matches!(cmds.first(), Some(&"install") | Some(&"search")) {
            Command::new("apt")
        } else {
            Self.cmd()
        }
    }
}

impl PackageManager for AdvancedPackageTool {
    fn pkg_delimiter(&self) -> char {
        '='
    }

    fn parse_pkg<'a>(&self, line: &str) -> Option<Package<'a>> {
        let Some((name, info)) = line.split_once('/') else {
            return None;
        };
        if matches!(info.split_whitespace().count(), 3 | 4) {
            let ver = info.split_whitespace().nth(1)?;
            Some(Package::from(name.to_owned()).with_version(ver.to_owned()))
        } else {
            None
        }
    }

    fn add_repo(&self, repo: &str) -> Result<(), RepoError> {
        let mut sources = fs::File::options()
            .append(true)
            .open(Self::SOURCES)
            .map_err(RepoError::new)?;
        sources
            .write_fmt(format_args!("\n{}", repo))
            .map_err(RepoError::new)
    }
}

impl Display for AdvancedPackageTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AdvancedPackageTool (APT)")
    }
}

impl Commands for AdvancedPackageTool {
    fn cmd(&self) -> Command {
        Command::new("apt-get")
    }
    fn get_cmds(&self, cmd: Cmd) -> &'static [&'static str] {
        match cmd {
            Cmd::Install => &["install"],
            Cmd::Uninstall => &["remove"],
            Cmd::Update => &["install"],
            Cmd::UpdateAll => &["upgrade"],
            Cmd::List => &["list"],
            Cmd::Sync => &["update"],
            Cmd::AddRepo => &[],
            Cmd::Search => &["search"],
        }
    }
    fn get_flags(&self, cmd: Cmd) -> &'static [&'static str] {
        match cmd {
            Cmd::Install | Cmd::Uninstall | Cmd::UpdateAll => &["--yes"],
            Cmd::Update => &["--yes", "--only-upgrade"],
            Cmd::List => &["--installed"],
            _ => &[],
        }
    }

    fn exec_cmds(&self, cmds: &[&str]) -> std::process::Output {
        Self::alt_cmd(cmds).args(cmds).output().unwrap()
    }

    fn exec_cmds_status(&self, cmds: &[&str]) -> std::process::ExitStatus {
        Self::alt_cmd(cmds).args(cmds).status().unwrap()
    }

    fn exec_cmds_spawn(&self, cmds: &[&str]) -> std::process::Child {
        Self::alt_cmd(cmds).args(cmds).spawn().unwrap()
    }
}
