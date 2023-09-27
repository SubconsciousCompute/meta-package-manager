use crate::{Commands, PackageManager, SubCommand};

pub struct HomeBrew;

impl HomeBrew {
    pub fn new() -> Self {
        HomeBrew
    }
}

impl PackageManager for HomeBrew {
    fn name(&self) -> &'static str {
        "HomeBrew"
    }

    fn pkg_delimiter(&self) -> char {
        '@'
    }
}

impl Commands for HomeBrew {
    fn cmd(&self) -> &'static str {
        "brew"
    }
    fn sub_cmd(&self, sub_cmd: SubCommand) -> &'static str {
        match sub_cmd {
            SubCommand::Install => "install",
            SubCommand::Uninstall => "uninstall",
            SubCommand::Update | SubCommand::UpdateAll => "upgrade",
            SubCommand::List => "list",
            SubCommand::Sync => "update",
            SubCommand::AddRepo => "tap",
            SubCommand::Search => "search",
        }
    }
}
