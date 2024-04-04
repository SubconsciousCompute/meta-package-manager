//! mpm library

use std::{
    borrow::Cow,
    error::Error,
    fmt::{Debug, Display},
    process::{Child, Command, ExitStatus, Output},
};

pub mod managers;
pub mod utils;

#[cfg(feature = "verify")]
pub mod verify;

#[cfg(test)]
mod libtests;

/// Primary interface for implementing a package manager
///
/// Multiple package managers can be grouped together as dyn PackageManager.
pub trait PackageManager: Commands + Debug + Display {
    /// Defines a delimeter to use while formatting package name and version
    ///
    /// For example, HomeBrew supports `<name>@<version>` and APT supports
    /// `<name>=<version>`. Their appropriate delimiters would be '@' and
    /// '=', respectively. For package managers that require additional
    /// formatting, overriding the default trait methods would be the way to go.
    fn pkg_delimiter(&self) -> char;

    /// Get a formatted string of the package as <name><delimiter><version>
    ///
    /// Note: this functions returns a formatted string only if version
    /// information is present. Otherwise, only a borrowed name string is
    /// returned. Which is why this function returns a 'Cow<str>' and not a
    /// `String`.
    fn pkg_format<'a>(&self, pkg: &'a Package) -> Cow<'a, str> {
        if let Some(v) = pkg.version() {
            format!("{}{}{}", pkg.name, self.pkg_delimiter(), v).into()
        } else {
            pkg.name().into()
        }
    }
    /// Returns a package after parsing a line of stdout output from the
    /// underlying package manager.
    ///
    /// This method is internally used in other default methods like
    /// [``PackageManager::search``] to parse packages from the output.
    ///
    /// The default implementation merely tries to split the line at the
    /// provided delimiter (see [``PackageManager::pkg_delimiter``])
    /// and trims spaces. It returns a package with version information on
    /// success, or else it returns a package with only a package name.
    /// For package maangers that have unusual or complex output, users are free
    /// to override this method. Note: Remember to construct a package with
    /// owned values in this method.
    fn parse_pkg<'a>(&self, line: &str) -> Option<Package<'a>> {
        let pkg = if let Some((name, version)) = line.split_once(self.pkg_delimiter()) {
            Package::from(name.trim().to_owned()).with_version(version.trim().to_owned())
        } else {
            Package::from(line.trim().to_owned())
        };
        Some(pkg)
    }

    /// Parses output, generally from stdout, to a Vec of Packages.
    ///
    /// The default implementation uses [``PackageManager::parse_pkg``] for
    /// parsing each line into a [`Package`].
    fn parse_output(&self, out: &[u8]) -> Vec<Package> {
        let outstr = String::from_utf8_lossy(out);
        outstr
            .lines()
            .filter_map(|s| {
                let ts = s.trim();
                if !ts.is_empty() {
                    self.parse_pkg(ts)
                } else {
                    None
                }
            })
            .collect()
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

    /// Install a single package
    ///
    /// For multi-package operations, see [``PackageManager::exec_op``]
    fn install(&self, pkg: Package) -> ExitStatus {
        self.exec_op(&[pkg], Operation::Install)
    }

    /// Uninstall a single package
    ///
    /// For multi-package operations, see [``PackageManager::exec_op``]
    fn uninstall(&self, pkg: Package) -> ExitStatus {
        self.exec_op(&[pkg], Operation::Uninstall)
    }

    /// Update a single package
    ///
    /// For multi-package operations, see [``PackageManager::exec_op``]
    fn update(&self, pkg: Package) -> ExitStatus {
        self.exec_op(&[pkg], Operation::Update)
    }

    /// List installed packages
    fn list_installed(&self) -> Vec<Package> {
        let out = self.exec_cmds(&self.consolidated(Cmd::List, &[]));
        self.parse_output(&out.stdout)
    }

    /// Execute an operation on multiple packages, such as install, uninstall
    /// and update
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
    ///
    /// Since the implementation might greatly vary among different package
    /// managers this method returns a `Result` instead of the usual
    /// `ExitStatus`.
    fn add_repo(&self, repo: &str) -> Result<(), RepoError> {
        let cmds = self.consolidated(Cmd::AddRepo, &[repo]);
        self.exec_cmds_status(&cmds)
            .success()
            .then_some(())
            .ok_or(RepoError::default())
    }
}

/// Error type for indicating failure in [``PackageManager::add_repo``]
///
/// Use [``RepoError::default``] when no meaningful source of the error is
/// available.
#[derive(Default, Debug)]
pub struct RepoError {
    pub source: Option<Box<dyn Error + 'static>>,
}

impl RepoError {
    /// Construct `RepoError` with underlying error source/cause
    ///
    /// Use [``RepoError::default``] when no meaningful source of the error is
    /// available.
    pub fn new<E: Error + 'static>(source: E) -> Self {
        Self {
            source: Some(Box::new(source)),
        }
    }

    /// Construct 'RepoError' with an error message set as its error source
    ///
    /// Use [``RepoError::new``] to wrap an existing error.
    /// Use [``RepoError::default``] when no meaningful source of the error is
    /// available.
    pub fn with_msg(msg: &'static str) -> Self {
        Self {
            source: Some(msg.into()),
        }
    }
}

impl Display for RepoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(s) = self.source() {
            f.write_fmt(format_args!("failed to add repo: {}", s))
        } else {
            f.write_str("failed to add repo")
        }
    }
}

impl Error for RepoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_deref()
    }
}

/// Trait for defining package panager commands in one place
///
/// Only [``Commands::cmd``] and [``Commands::commands``] are required, the rest
/// are simply conviniece methods that internally call [``Commands::commands``].
/// The trait [``PackageManager``] depends on this to provide default
/// implementations.
pub trait Commands {
    /// Primary command of the package manager. For example, 'brew', 'apt', and
    /// 'dnf', constructed with [``std::process::Command::new``].
    fn cmd(&self) -> Command;
    /// Returns the appropriate command/s for the given supported command type.
    /// Check [``Cmd``] enum to see all supported commands.
    fn get_cmds(&self, cmd: Cmd) -> &'static [&'static str];
    /// Returns the appropriate flags for the given command type. Check
    /// [``Cmd``] enum to see all supported commands.
    ///
    /// Flags are optional, which is why the default implementation returns an
    /// empty slice
    fn get_flags(&self, _cmd: Cmd) -> &'static [&'static str] {
        &[]
    }
    /// Retreives defined commands and flags for the given [``Cmd``] type and
    /// returns a Vec of args in the order: `[commands..., user-args...,
    /// flags...]`
    ///
    /// The appropriate commands and flags are determined with the help of the
    /// enum [``Cmd``] For finer control, a general purpose function
    /// [``consolidated_args``] is also provided.
    #[inline]
    fn consolidated<'a>(&self, cmd: Cmd, args: &[&'a str]) -> Vec<&'a str> {
        let commands = self.get_cmds(cmd);
        let flags = self.get_flags(cmd);
        let mut vec = Vec::with_capacity(commands.len() + flags.len() + args.len());
        vec.extend(
            commands
                .iter()
                .chain(args.iter())
                .chain(flags.iter())
                .copied(),
        );
        vec
    }
    /// Run arbitrary commands against the package manager command and get
    /// output
    ///
    /// # Panics
    /// This fn can panic when the defined [``Commands::cmd``] is not found in
    /// path. This can be avoided by using [``verified::Verified``]
    /// or manually ensuring that the [``Commands::cmd``] is valid.
    fn exec_cmds(&self, cmds: &[&str]) -> Output {
        tracing::info!("Executing {:?} with args {:?}", self.cmd(), cmds);
        self.cmd()
            .args(cmds)
            .output()
            .expect("command executed without a prior check")
    }
    /// Run arbitrary commands against the package manager command and wait for
    /// ExitStatus
    ///
    /// # Panics
    /// This fn can panic when the defined [``Commands::cmd``] is not found in
    /// path. This can be avoided by using [``verified::Verified``]
    /// or manually ensuring that the [``Commands::cmd``] is valid.
    fn exec_cmds_status(&self, cmds: &[&str]) -> ExitStatus {
        self.cmd()
            .args(cmds)
            .status()
            .expect("command executed without a prior check")
    }
    /// Run arbitrary commands against the package manager command and return
    /// handle to the spawned process
    ///
    /// # Panics
    /// This fn can panic when the defined [``Commands::cmd``] is not found in
    /// path. This can be avoided by using [``verified::Verified``]
    /// or manually ensuring that the [``Commands::cmd``] is valid.
    fn exec_cmds_spawn(&self, cmds: &[&str]) -> Child {
        self.cmd()
            .args(cmds)
            .spawn()
            .expect("command executed without a prior check")
    }
}

/// Representation of a package manager command
///
/// All the variants are the type of commands that a type that imlements
/// [``Commands``] and [``PackageManager``] (should) support.
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
/// It can be constructed with any type that implements `Into<Cow<sr>>`, for
/// example, `&str` and `String`. `Package::from("python")` or with version,
/// `Package::from("python").with_version("3.10.0")`.
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

/// Operation type to execute using [``Package::exec_op``]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    Install,
    Uninstall,
    Update,
}

/// General purpose version of [``Commands::consolidated``] for consolidating
/// different types of arguments into a single Vec
#[inline]
pub fn consolidate_args<'a>(cmds: &[&'a str], args: &[&'a str], flags: &[&'a str]) -> Vec<&'a str> {
    let mut vec = Vec::with_capacity(cmds.len() + args.len() + flags.len());
    vec.extend(cmds.iter().chain(args.iter()).chain(flags.iter()).copied());
    vec
}
