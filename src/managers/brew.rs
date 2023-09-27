use crate::{Error, Operation, PackError, Package, PackageManager, Repo};

pub struct HomeBrew;

impl HomeBrew {
    const NAME: &'static str = "HomeBrew";
    const CMD: &'static str = "brew";
    const LIST_CMD: &'static str = "list";
    const SEARCH_CMD: &'static str = "search";
    const INSTALL_CMD: &'static str = "install";
    const UNINSTALL_CMD: &'static str = "uninstall";
    const UPDATE_CMD: &'static str = "update";
    const REPO_CMD: &'static str = "tap";

    pub fn new() -> Self {
        HomeBrew
    }

    fn parse_package<'a, 'b>(line: &'a str) -> Package<'b> {
        if let Some((name, version)) = line.split_once('@') {
            return Package::from(name.trim().to_owned()).with_version(version.trim().to_owned());
        }
        Package::from(line.trim().to_owned())
    }
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
        // TODO evaluate whether this error should be handled
        let outstr = std::str::from_utf8(&out.stdout).unwrap();
        outstr.lines().map(|s| Self::parse_package(s)).collect()
    }

    fn execute_op(&self, pack: &Package, op: Operation) -> PackError<()> {
        let cmd = match op {
            Operation::Install => Self::INSTALL_CMD,
            Operation::Uninstall => Self::UNINSTALL_CMD,
            Operation::Update => Self::UPDATE_CMD,
        };
        self.execute_cmds_status(&[cmd, &pack.fmt_with_delimiter('@')])
            .success()
            .then_some(())
            .ok_or(Error)
    }

    fn list_installed(&self) -> Vec<Package> {
        let out = self.execute_cmds(&[Self::LIST_CMD]);
        let outstr = std::str::from_utf8(&out.stdout).unwrap();
        outstr.lines().map(|s| Self::parse_package(s)).collect()
    }

    fn add_repo(&self, repo: Repo) -> PackError<()> {
        self.execute_cmds_status(&[Self::REPO_CMD, repo.as_str()])
            .success()
            .then_some(())
            .ok_or(Error)
    }
}
