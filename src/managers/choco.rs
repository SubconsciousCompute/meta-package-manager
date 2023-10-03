use std::borrow::Cow;

use crate::{Cmd, Commands, Package, PackageManager};

pub struct Chocolatey;

impl PackageManager for Chocolatey {
    fn name(&self) -> &'static str {
        "Chocolatey"
    }
    fn pkg_delimiter(&self) -> char {
        '|'
    }
    fn pkg_format<'a>(&self, pkg: &'a Package) -> Cow<'a, str> {
        if let Some(v) = pkg.version() {
            format!("{} --version {}", pkg.name, v).into()
        } else {
            pkg.name().into()
        }
    }
}

impl Commands for Chocolatey {
    fn cmd(&self) -> &'static str {
        "choco"
    }
    fn command(&self, cmd: Cmd) -> &'static [&'static str] {
        match cmd {
            Cmd::Install => &["install"],
            Cmd::Uninstall => &["uninstall"],
            Cmd::Update => &["upgrade"],
            Cmd::UpdateAll => &["upgrade", "all"],
            Cmd::List => &["list"],
            // Since chocolatey does not have an analogue for sync command
            // updating chocolatey was chosen as an alternative
            Cmd::Sync => &["upgrade", "chocolatey"],
            Cmd::AddRepo => &["source", "add"],
            Cmd::Search => &["search"],
        }
    }
    fn flags(&self, cmd: Cmd) -> &'static [&'static str] {
        match cmd {
            Cmd::List | Cmd::Search => &["--limit-output"],
            Cmd::Install | Cmd::Update | Cmd::UpdateAll => &["--yes"],
            _ => &[],
        }
    }
}
