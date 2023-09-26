use std::{
    borrow::Cow,
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
    version: Option<Version>,
}

impl Package<'_> {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn version(&self) -> Option<&Version> {
        self.version.as_ref()
    }
    pub fn with_version(mut self, ver: Version) -> Self {
        self.version.replace(ver);
        self
    }
}

impl<'a> From<&'a str> for Package<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            name: value.into(),
            version: None,
        }
    }
}

impl From<String> for Package<'_> {
    fn from(value: String) -> Self {
        Self {
            name: value.into(),
            version: None,
        }
    }
}

#[derive(Debug)]
pub struct Version(u8, u8, u8);

pub enum Operation {
    Install,
    Uninstall,
    Update,
}
