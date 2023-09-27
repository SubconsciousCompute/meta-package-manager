use std::{
    borrow::Cow,
    fmt::Display,
    process::{Command, ExitStatus, Output, Stdio},
};

use url::Url as ParsedUrl;

pub mod brew;

/// Primary interface for implementing a package manager
///
/// Multiple package managers can be grouped together as dyn PackageManager.
pub trait PackageManager {
    /// Package manager name
    fn name(&self) -> &'static str;

    /// Package manager command
    fn cmd(&self) -> &'static str;

    /// Check if package manager is installed on the system
    fn is_installed(&self) -> bool {
        Command::new(self.cmd())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok()
    }

    /// General package search
    fn search(&self, pack: &str) -> Vec<Package>;

    /// List installed packages
    fn list_installed(&self) -> Vec<Package>;

    /// Execute operation on a package, such as install, uninstall and update
    fn execute_op(&self, pack: &Package, op: Operation) -> PackError<()>;

    /// Run arbitrary commands against the package manager and get output
    fn execute_cmds(&self, cmds: &[&str]) -> Output {
        // safe to unwrap when package manager is known to be available (see is_installed fn)
        Command::new(self.cmd()).args(cmds).output().unwrap()
    }

    /// Run arbitrary commands against the package manager and wait for ExitStatus
    fn execute_cmds_status(&self, cmds: &[&str]) -> ExitStatus {
        // safe to unwrap when package manager is known to be available (see is_installed fn)
        Command::new(self.cmd()).args(cmds).status().unwrap()
    }

    /// Add third-party repository to the package manager's repository list
    fn add_repo(&self, repo: Repo) -> PackError<()>;
}

/// Temporary error type
pub struct Error;

/// Temporary error type alias
pub type PackError<T> = Result<T, Error>;

/// A representation of a package
///
/// This struct contains package's name and version information (optional).
/// It can be constructed with any type that implements `Into<Cow<sr>>`, for example, `&str` and `String`.
/// `Package::from("python")` or with version, `Package::from("python").with_version("3.10.0")`.
#[derive(Debug)]
pub struct Package<'a> {
    name: Cow<'a, str>,
    // Untyped version, might be replaced with a strongly typed one
    version: Option<Cow<'a, str>>,
}

impl<'a> Package<'a> {
    /// Package name
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Check if package has version information.
    pub fn has_version(&self) -> bool {
        self.version.is_some()
    }
    /// Get version information if present
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    /// Add or replace package's version
    pub fn with_version<V>(mut self, ver: V) -> Self
    where
        V: Into<Cow<'a, str>>,
    {
        self.version.replace(ver.into());
        self
    }

    /// Get a formatted string of the package as <name><delimiter><version>
    ///
    /// Note: this functions returns a formatted string only if version information is present.
    /// Otherwise, only a borrowed name string is returned. Which is why this function returns a 'Cow<str>' and not a `String`.
    pub fn fmt_with_delimiter(&self, delimiter: char) -> Cow<str> {
        if let Some(v) = self.version() {
            format!("{}{}{}", self.name, delimiter, v).into()
        } else {
            self.name().into()
        }
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
            // might be changed later for a better format
            write!(f, "{}@{}", self.name, v)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

/// Operation type to execute using [``Package::execute_op``]
pub enum Operation {
    Install,
    Uninstall,
    Update,
}

pub enum Repo<'a> {
    Url(Url),
    Other(&'a str),
}

impl Repo<'_> {
    pub fn as_str(&self) -> &str {
        match self {
            Repo::Url(u) => u.as_str(),
            Repo::Other(o) => o,
        }
    }
}

/// A strongly typed URL to ensure URL validity
///
/// This struct merely is a wrapper for Url from the url crate
/// and exposes only the necessarry functionality.
pub struct Url(ParsedUrl);

impl Url {
    /// Parse string into URL
    pub fn parse(url: &str) -> PackError<Url> {
        let parsed = ParsedUrl::parse(url).map_err(|_| Error)?;
        Ok(Url(parsed))
    }
    /// Get parsed URL as &str
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
