use crate::{Cmd, Commands, PackageManager};

#[derive(Debug)]
pub struct HomeBrew;

impl HomeBrew {
    pub fn new() -> Self {
        HomeBrew
    }
}

impl PackageManager for HomeBrew {
    fn pkg_delimiter(&self) -> char {
        '@'
    }
}

impl Commands for HomeBrew {
    fn cmd(&self) -> &'static str {
        "brew"
    }
    fn command(&self, cmd: Cmd) -> &'static [&'static str] {
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
