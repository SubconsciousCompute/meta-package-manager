use crate::{Cmd, Commands, PackageManager};

pub struct Chocolatey;

impl PackageManager for Chocolatey {
    fn name(&self) -> &'static str {
        "Chocolatey"
    }
    fn pkg_delimiter(&self) -> char {
        ' '
    }
}

impl Commands for Chocolatey {
    fn cmd(&self) -> &'static str {
        "choco"
    }
    fn command(&self, cmd: crate::Cmd) -> &'static [&'static str] {
        match cmd {
            Cmd::Install => &["install"],
            Cmd::Uninstall => &["uninstall"],
            Cmd::Update => &["upgrade"],
            Cmd::UpdateAll => &["upgrade", "all"],
            Cmd::List => &["list"],
            Cmd::Sync => &["sync"],
            Cmd::AddRepo => &["source", "add"],
            Cmd::Search => &["search"],
        }
    }
    fn flags(&self, cmd: Cmd) -> &'static [&'static str] {
        match cmd {
            Cmd::List => &["--local"],
            Cmd::Install | Cmd::Update | Cmd::UpdateAll => &["-y"],
            _ => &[],
        }
    }
}
