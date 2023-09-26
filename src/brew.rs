use super::{Error, Operation, PackError, Package, PackageManager};

// Example implementation of HomeBrew package manager
pub struct HomeBrew;

impl HomeBrew {
    const NAME: &'static str = "HomeBrew";
    const CMD: &'static str = "brew";
    const LIST_CMD: &'static str = "list";
    const SEARCH_CMD: &'static str = "search";
    const INSTALL_CMD: &'static str = "install";
    const UNINSTALL_CMD: &'static str = "uninstall";
    const UPDATE_CMD: &'static str = "update";
}

impl PackageManager for HomeBrew {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn cmd(&self) -> &'static str {
        Self::CMD
    }

    fn search(&self, pack: &str) -> Vec<Package> {
        let out = self.execute_cmds(&[Self::SEARCH_CMD, pack]);
        let outstr = std::str::from_utf8(&out.stdout).unwrap();
        outstr
            .lines()
            .map(|s| Package::from(s.to_owned()))
            .collect()
    }

    fn execute_op(&self, pack: &Package, op: Operation) -> PackError<()> {
        let cmd = match op {
            Operation::Install => Self::INSTALL_CMD,
            Operation::Uninstall => Self::UNINSTALL_CMD,
            Operation::Update => Self::UPDATE_CMD,
        };
        self.execute_cmds_status(&[cmd, pack.name()])
            .success()
            .then_some(())
            .ok_or(Error)
    }

    fn list_installed(&self) -> Vec<Package> {
        let out = self.execute_cmds(&[Self::LIST_CMD]);
        let outstr = std::str::from_utf8(&out.stdout).unwrap();
        outstr
            .lines()
            .map(|s| Package::from(s.to_owned()))
            .collect()
    }
}
