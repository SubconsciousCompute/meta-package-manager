use std::{fmt::Display, process::Command};

use crate::{Cmd, Commands, PackageManager};

/// Wrapper for the Homebrew package manager.
///
/// [Homebrew — The Missing Package Manager for macOS (or Linux)](https://brew.sh/)
#[derive(Debug)]
pub struct Homebrew;

impl PackageManager for Homebrew {
    fn pkg_delimiter(&self) -> char {
        '@'
    }
}

impl Commands for Homebrew {
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

impl Display for Homebrew {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Homebrew")
    }
}
