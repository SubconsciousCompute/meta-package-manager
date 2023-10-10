use crate::{Cmd, Commands, Package, PackageManager, RepoError};
use std::{fmt::Display, fs, io::Write, process::Command};

#[derive(Debug)]
pub struct AdvancedPackageTool;

impl AdvancedPackageTool {
    const SOURCES: &str = "/etc/apt/sources.list";
}

impl PackageManager for AdvancedPackageTool {
    fn pkg_delimiter(&self) -> char {
        '='
    }

    fn parse_pkg<'a>(&self, line: &str) -> Option<Package<'a>> {
        let Some((name, info)) = line.split_once('/') else {
            return None;
        };
        let ver = info.split_whitespace().nth(1)?;
        Some(Package::from(name.to_owned()).with_version(ver.to_owned()))
    }

    fn list_installed(&self) -> Vec<Package> {
        let out = Command::new("apt")
            .args(&self.consolidated(Cmd::List, &[]))
            .output()
            .unwrap();
        self.parse_output(&out.stdout)
    }

    fn search(&self, pack: &str) -> Vec<Package> {
        let out = Command::new("apt")
            .args(&self.consolidated(Cmd::Search, &[pack]))
            .output()
            .unwrap();
        self.parse_output(&out.stdout)
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
}
