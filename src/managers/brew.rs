use std::{fmt::Display, process::Command};

use crate::{Cmd, Commands, PackageManager};

#[derive(Debug)]
pub struct HomeBrew;

impl PackageManager for HomeBrew {
    fn pkg_delimiter(&self) -> char {
        '@'
    }
}

impl Commands for HomeBrew {
    fn cmd(&self) -> Command {
        Command::new("brew")
    }
    fn get_cmds(&self, cmd: Cmd) -> &'static [&'static str] {
        match cmd {
            Cmd::Install => &["install"],
            Cmd::Uninstall => &["uninstall"],
            Cmd::Update | Cmd::UpdateAll => &["upgrade"],
            Cmd::List => &["list"],
            Cmd::Sync => &["update"],
            Cmd::AddRepo => &["tap"],
            Cmd::Search => &["search"],
        }
    }
}

impl Display for HomeBrew {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HomeBrew")
    }
}
