use crate::{Cmd, Commands, Package, PackageManager};
use std::{fmt::Display, process::Command};

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
            Cmd::AddRepo => &["config-manager", "--add-repo"], // must come before repo
            Cmd::Search => &["search"],
        }
    }

    fn get_flags(&self, cmd: Cmd) -> &'static [&'static str] {
        match cmd {
            Cmd::Install | Cmd::Uninstall | Cmd::Update | Cmd::UpdateAll => &["--yes"],
            Cmd::List => &["--installed"],
            Cmd::Search => &["-q"],
            _ => &[],
        }
    }
}
