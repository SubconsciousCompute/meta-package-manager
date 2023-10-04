use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    process::{Child, Command, ExitStatus, Output, Stdio},
};

use url::Url as ParsedUrl;

pub mod managers;

#[cfg(test)]
mod libtests;

/// Primary interface for implementing a package manager
///
/// Multiple package managers can be grouped together as dyn PackageManager.
pub trait PackageManager: Commands + Debug {
    /// Defines a delimeter to use while formatting package name and version
    ///
    /// For example, HomeBrew supports `<name>@<version>` and APT supports `<name>=<version>`.
    /// Their appropriate delimiters would be '@' and '=', respectively.
    /// For package managers that require additional formatting, overriding the default trait methods would be the way to go.
    fn pkg_delimiter(&self) -> char;

    /// Get a formatted string of the package as <name><delimiter><version>
    ///
    /// Note: this functions returns a formatted string only if version information is present.
    /// Otherwise, only a borrowed name string is returned. Which is why this function returns a 'Cow<str>' and not a `String`.
    fn pkg_format<'a>(&self, pkg: &'a Package) -> Cow<'a, str> {
        if let Some(v) = pkg.version() {
            format!("{}{}{}", pkg.name, self.pkg_delimiter(), v).into()
        } else {
            pkg.name().into()
        }
    }
    /// Returns a package after parsing a line of stdout output from the underlying package manager.
    ///
    /// This method is internally used in other default methods like [``PackageManager::search``] to parse packages from the output.
    ///
    /// The default implementation merely tries to split the line at the provided delimiter (see [``PackageManager::pkg_delimiter``])
    /// and trims spaces. It returns a package with version information on success, or else it returns a package with only a package name.
    /// For package maangers that have unusual or complex output, users are free to override this method. Note: Remember to construct a package with owned values in this method.
    fn parse<'a, 'b>(&self, line: &'a str) -> Package<'b> {
        if let Some((name, version)) = line.split_once(self.pkg_delimiter()) {
            return Package::from(name.trim().to_owned()).with_version(version.trim().to_owned());
        }
        Package::from(line.trim().to_owned())
    }

    /// Parses output, generally from stdout, to a Vec of Packages.
    ///
    /// The default implementation uses [``PackageManager::parse``] for parsing each line into a [`Package`].
    fn parse_output(&self, out: &[u8]) -> Vec<Package> {
        let outstr = std::str::from_utf8(out).unwrap();
        outstr
            .lines()
            .filter_map(|s| {
                let ts = s.trim();
                (!ts.is_empty()).then_some(self.parse(ts))
            })
            .collect()
    }

    /// Check if package manager is installed on the system
    fn is_installed(&self) -> bool {
        Command::new(self.cmd())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .is_ok()
    }

    /// General package search
    fn search(&self, pack: &str) -> Vec<Package> {
        let cmds = self.consolidated(Cmd::Search, &[pack]);
        let out = self.exec_cmds(&cmds);
        self.parse_output(&out.stdout)
    }

    /// Sync package manaager repositories
    fn sync(&self) -> ExitStatus {
        self.exec_cmds_status(&self.consolidated(Cmd::Sync, &[]))
    }

    /// Update/upgrade all packages
    fn update_all(&self) -> ExitStatus {
        self.exec_cmds_status(&self.consolidated(Cmd::UpdateAll, &[]))
    }

    /// List installed packages
    fn list_installed(&self) -> Vec<Package> {
        let out = self.exec_cmds(&self.consolidated(Cmd::List, &[]));
        self.parse_output(&out.stdout)
    }

    /// exec an operation on multiple packages, such as install, uninstall and update
    fn exec_op(&self, pkgs: &[Package], op: Operation) -> ExitStatus {
        let command = match op {
            Operation::Install => Cmd::Install,
            Operation::Uninstall => Cmd::Uninstall,
            Operation::Update => Cmd::Update,
        };
        let fmt: Vec<_> = pkgs.iter().map(|p| self.pkg_format(p)).collect();
        let cmds = self.consolidated(
            command,
            &fmt.iter().map(|v| v.as_ref()).collect::<Vec<&str>>(),
        );
        self.exec_cmds_status(&cmds)
    }

    /// Add third-party repository to the package manager's repository list
    fn add_repo(&self, repo: Repo) -> ExitStatus {
        let cmds = self.consolidated(Cmd::AddRepo, &[repo.as_str()]);
        self.exec_cmds_status(&cmds)
    }

    /// Run arbitrary commands against the package manager and get output
    fn exec_cmds(&self, cmds: &[&str]) -> Output {
        // safe to unwrap when package manager is known to be available (see is_installed fn)
        Command::new(self.cmd()).args(cmds).output().unwrap()
    }

    /// Run arbitrary commands against the package manager and wait for ExitStatus
    fn exec_cmds_status(&self, cmds: &[&str]) -> ExitStatus {
        // safe to unwrap when package manager is known to be available (see is_installed fn)
        Command::new(self.cmd()).args(cmds).status().unwrap()
    }

    /// Run arbitrary commands against the package manager and return handle to the spawned process
    fn exec_cmds_spawn(&self, cmds: &[&str]) -> Child {
        // safe to unwrap when package manager is known to be available (see is_installed fn)
        Command::new(self.cmd()).args(cmds).spawn().unwrap()
    }
}

/// Trait for defining package panager commands in one place
///
/// Only [``Commands::cmd``] and [``Commands::commands``] are required, the rest are simply conviniece methods
/// that internally call [``Commands::commands``]. The trait [``PackageManager``] depends on this to provide default implementations.
pub trait Commands {
    /// Primary command of the package manager. For example, 'brew', 'apt', and 'dnf'.
    fn cmd(&self) -> &'static str;
    /// Returns the appropriate command/s for the given supported command type. Check [``Cmd``] enum to see all supported commands.
    fn command(&self, cmd: Cmd) -> &'static [&'static str];
    /// Returns the appropriate flags for the given command type. Check [``Cmd``] enum to see all supported commands.
    ///
    /// Flags are optional, which is why the default implementation returns an empty slice
    fn flags(&self, _cmd: Cmd) -> &'static [&'static str] {
        &[]
    }
    /// Retreives defined commands and flags for the given [``Cmd``] type and returns a Vec of args in the order: `[commands..., user-args..., flags..., user-flags...]`
    ///
    /// This is an extended version of [``Commands::consolidated``], which only supports user args, and no flags.
    /// The appropriate commands and flags are determined with the help of the enum [``Cmd``]
    /// For finer control, a general purpose function [``consolidated_args``] is also provided.
    #[inline]
    fn consolidated_ext<'a>(&self, cmd: Cmd, args: &[&'a str], flags: &[&'a str]) -> Vec<&'a str> {
        let commands = self.command(cmd);
        let default_flags = self.flags(cmd);
        let mut vec =
            Vec::with_capacity(commands.len() + flags.len() + args.len() + default_flags.len());
        vec.extend(
            commands
                .iter()
                .chain(args.iter())
                .chain(default_flags.iter())
                .chain(flags.iter())
                .map(|e| *e),
        );
        vec
    }
    /// Retreives defined commands and flags for the given [``Cmd``] type and returns a Vec of args in the order: `[commands..., user-args..., flags...]`
    ///
    /// The appropriate commands and flags are determined with the help of the enum [``Cmd``]
    /// For supplying additional flags in addition to default ones, see [``Commands::consolidated_ext``]
    /// For finer control, a general purpose function [``consolidated_args``] is also provided.
    #[inline]
    fn consolidated<'a>(&self, cmd: Cmd, args: &[&'a str]) -> Vec<&'a str> {
        self.consolidated_ext(cmd, args, &[])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cmd {
    Install,
    Uninstall,
    Update,
    UpdateAll,
    List,
    Sync,
    AddRepo,
    Search,
}

/// A representation of a package
///
/// This struct contains package's name and version information (optional).
/// It can be constructed with any type that implements `Into<Cow<sr>>`, for example, `&str` and `String`.
/// `Package::from("python")` or with version, `Package::from("python").with_version("3.10.0")`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
            write!(f, "{} - {}", self.name, v)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

/// Operation type to exec using [``Package::exec_op``]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    Install,
    Uninstall,
    Update,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Url(ParsedUrl);

impl Url {
    /// Parse string into URL
    pub fn parse(url: &str) -> Result<Url, url::ParseError> {
        ParsedUrl::parse(url).map(Url)
    }
    /// Get parsed URL as &str
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// General purpose version of [``Commands::consolidated``] for consolidating different types of arguments into a single Vec
#[inline]
pub fn consolidate_args<'a>(cmds: &[&'a str], args: &[&'a str], flags: &[&'a str]) -> Vec<&'a str> {
    let mut vec = Vec::with_capacity(cmds.len() + args.len() + flags.len());
    vec.extend(
        cmds.iter()
            .chain(args.iter())
            .chain(flags.iter())
            .map(|e| *e),
    );
    vec
}
