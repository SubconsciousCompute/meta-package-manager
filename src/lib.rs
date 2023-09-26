use std::process::{Command, ExitStatus, Output, Stdio};

// Module containing example implementation of HomeBrew
mod brew;

// Primary interface. Multiple package managers can be grouped together as dyn PackageManager.
trait PackageManager {
    // Package manager name
    fn name(&self) -> &'static str;

    // Package manager command
    fn cmd(&self) -> &'static str;

    // Check if package manager is installed on system
    fn is_installed(&self) -> bool {
        Command::new(self.cmd())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok()
    }

    // General package search
    fn search(&self, pack: &str) -> Vec<Package>;

    // List installed packages
    fn list_installed(&self) -> Vec<Package>;

    // Install, uninstall and update
    fn execute(&self, pack: &Package, op: Operation) -> PackError<()>;

    // Run arbitrary commands against the package manager and get output
    fn execute_cmds(&self, cmds: &[&str]) -> Output {
        // safe to unwrap when package manager is known to be available (see is_installed fn)
        Command::new(self.cmd()).args(cmds).output().unwrap()
    }

    // Run arbitrary commands against the package manager and wait for ExitStatus
    fn execute_cmds_status(&self, cmds: &[&str]) -> ExitStatus {
        // safe to unwrap when package manager is known to be available (see is_installed fn)
        Command::new(self.cmd()).args(cmds).status().unwrap()
    }
}

struct Error;

type PackError<T> = Result<T, Error>;

#[derive(Debug)]
struct Package {
    name: String,
    version: Option<Version>,
impl Package {
    pub fn new(name: String) -> Self {
        Package {
            name,
            version: None,
        }
    }
    pub fn with_version(mut self, ver: Version) -> Self {
        self.version.replace(ver);
        self
    }
}

#[derive(Debug)]
struct Version(u8, u8, u8);

enum Operation {
    Install,
    Uninstall,
    Update,
}
