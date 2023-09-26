use std::{
    borrow::Cow,
    fmt::Display,
    process::{Command, ExitStatus, Output, Stdio},
};

// Module containing example implementation of HomeBrew
pub mod brew;

// Primary interface. Multiple package managers can be grouped together as dyn PackageManager.
pub trait PackageManager {
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
    fn execute_op(&self, pack: &Package, op: Operation) -> PackError<()>;

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

pub struct Error;

pub type PackError<T> = Result<T, Error>;

#[derive(Debug)]
pub struct Package<'a> {
    name: Cow<'a, str>,
    // Temporary untyped version
    version: Option<Cow<'a, str>>,
}

impl<'a> Package<'a> {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn has_version(&self) -> bool {
        self.version.is_some()
    }
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    pub fn with_version<V>(mut self, ver: V) -> Self
    where
        V: Into<Cow<'a, str>>,
    {
        self.version.replace(ver.into());
        self
    }
}

impl<'a, T> From<T> for Package<'a>
where
    T: Into<Cow<'a, str>>,
{
    fn from(value: T) -> Self {
        Self {
            name: value.into(),
            version: None,
        }
    }
}

impl Display for Package<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = self.version.as_ref() {
            write!(f, "{}@{}", self.name, v)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

pub enum Operation {
    Install,
    Uninstall,
    Update,
}
