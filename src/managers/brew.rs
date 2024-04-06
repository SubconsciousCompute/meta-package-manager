use std::{fmt::Display, process::Command};

use crate::{Cmd, PackageManager, PackageManagerCommands};

/// Wrapper for the Homebrew package manager.
///
/// [Homebrew — The Missing Package Manager for macOS (or Linux)](https://brew.sh/)
#[derive(Debug, Default)]
pub struct Homebrew;

impl PackageManager for Homebrew {
    fn pkg_delimiter(&self) -> char {
        '@'
    }
}

impl PackageManagerCommands for Homebrew {
    fn cmd(&self) -> Command {
        Command::new("brew")
    }

    fn get_cmds(&self, cmd: Cmd) -> Vec<String> {
        match cmd {
            Cmd::Install => vec!["install"],
            Cmd::Uninstall => vec!["uninstall"],
            Cmd::Update | Cmd::UpdateAll => vec!["upgrade"],
            Cmd::List => vec!["list"],
            Cmd::Sync => vec!["update"],
            Cmd::AddRepo => vec!["tap"],
            Cmd::Search => vec!["search"],
        }
        .iter()
        .map(|x| x.to_string())
        .collect()
    }
}

impl Display for Homebrew {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Homebrew")
    }
}
