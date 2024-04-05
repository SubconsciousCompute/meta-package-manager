//! Common types and traits.

use std::{error::Error, fmt::Display};

/// Primary interface for implementing a package manager
///
/// Multiple package managers can be grouped together as dyn PackageManager.
#[ambassador::delegatable_trait]
pub trait PackageManager: Commands + std::fmt::Debug + std::fmt::Display {
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
    fn pkg_format(&self, pkg: &Package) -> String {
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
    fn parse_pkg(&self, line: &str) -> Option<Package> {
        let pkg = if let Some((name, version)) = line.split_once(self.pkg_delimiter()) {
            Package::new(name.trim(), Some(version.trim()))
        } else {
            Package::new(line.trim(), None)
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
        let cmds = self.consolidated(crate::common::Cmd::Search, &[pack.to_string()]);
        let out = self.exec_cmds(&cmds);
        self.parse_output(&out.stdout)
    }

    /// Sync package manaager repositories
    fn sync(&self) -> std::process::ExitStatus {
        self.exec_cmds_status(&self.consolidated::<&str>(crate::common::Cmd::Sync, &[]))
    }

    /// Update/upgrade all packages
    fn update_all(&self) -> std::process::ExitStatus {
        self.exec_cmds_status(&self.consolidated::<&str>(crate::common::Cmd::UpdateAll, &[]))
    }

    /// Install a single package
    ///
    /// For multi-package operations, see [``PackageManager::exec_op``]
    fn install(&self, pkg: Package) -> std::process::ExitStatus {
        self.exec_op(&[pkg], Operation::Install)
    }

    /// Uninstall a single package
    ///
    /// For multi-package operations, see [``PackageManager::exec_op``]
    fn uninstall(&self, pkg: Package) -> std::process::ExitStatus {
        self.exec_op(&[pkg], Operation::Uninstall)
    }

    /// Update a single package
    ///
    /// For multi-package operations, see [``PackageManager::exec_op``]
    fn update(&self, pkg: Package) -> std::process::ExitStatus {
        self.exec_op(&[pkg], Operation::Update)
    }

    /// List installed packages
    fn list_installed(&self) -> Vec<Package> {
        let out = self.exec_cmds(&self.consolidated::<&str>(crate::common::Cmd::List, &[]));
        self.parse_output(&out.stdout)
    }

    /// Execute an operation on multiple packages, such as install, uninstall
    /// and update
    fn exec_op(&self, pkgs: &[Package], op: Operation) -> std::process::ExitStatus {
        let command = match op {
            Operation::Install => crate::common::Cmd::Install,
            Operation::Uninstall => crate::common::Cmd::Uninstall,
            Operation::Update => crate::common::Cmd::Update,
        };
        let fmt: Vec<_> = pkgs
            .iter()
            .map(|p| self.pkg_format(p).to_string())
            .collect();
        let cmds = self.consolidated(command, &fmt);
        self.exec_cmds_status(&cmds)
    }

    /// Add third-party repository to the package manager's repository list
    ///
    /// Since the implementation might greatly vary among different package
    /// managers this method returns a `Result` instead of the usual
    /// `std::process::ExitStatus`.
    fn add_repo(&self, repo: &str) -> anyhow::Result<()> {
        let cmds = self.consolidated(crate::common::Cmd::AddRepo, &[repo.to_string()]);
        let s = self.exec_cmds_status(&cmds);
        anyhow::ensure!(s.success(), "Error adding repo");
        Ok(())
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

/// Trait for defining package panager commands in one place
///
/// Only [``Commands::cmd``] and [``Commands::commands``] are required, the rest
/// are simply conviniece methods that internally call [``Commands::commands``].
/// The trait [``PackageManager``] depends on this to provide default
/// implementations.
#[ambassador::delegatable_trait]
pub trait Commands {
    /// Primary command of the package manager. For example, 'brew', 'apt', and
    /// 'dnf', constructed with [``std::process::Command::new``].
    fn cmd(&self) -> std::process::Command;

    /// Returns the appropriate command/s for the given supported command type.
    /// Check [``crate::common::Cmd``] enum to see all supported commands.
    fn get_cmds(&self, cmd: crate::common::Cmd) -> Vec<String>;

    /// Returns the appropriate flags for the given command type. Check
    /// [``crate::common::Cmd``] enum to see all supported commands.
    ///
    /// Flags are optional, which is why the default implementation returns an
    /// empty slice
    fn get_flags(&self, _cmd: crate::common::Cmd) -> Vec<String> {
        vec![]
    }

    /// Retreives defined commands and flags for the given
    /// [``crate::common::Cmd``] type and returns a Vec of args in the
    /// order: `[commands..., user-args..., flags...]`
    ///
    /// The appropriate commands and flags are determined with the help of the
    /// enum [``crate::common::Cmd``] For finer control, a general purpose
    /// function [``consolidated_args``] is also provided.
    #[inline]
    fn consolidated<S: AsRef<str>>(&self, cmd: crate::common::Cmd, args: &[S]) -> Vec<String> {
        let mut commands = self.get_cmds(cmd);
        commands.append(&mut args.into_iter().map(|x| x.as_ref().to_string()).collect());
        commands.append(&mut self.get_flags(cmd));
        commands
    }
    /// Run arbitrary commands against the package manager command and get
    /// output
    ///
    /// # Panics
    /// This fn can panic when the defined [``Commands::cmd``] is not found in
    /// path. This can be avoided by using [``verified::Verified``]
    /// or manually ensuring that the [``Commands::cmd``] is valid.
    fn exec_cmds(&self, cmds: &[String]) -> std::process::Output {
        tracing::info!("Executing {:?} with args {:?}", self.cmd(), cmds);
        self.cmd()
            .args(cmds)
            .output()
            .expect("command executed without a prior check")
    }
    /// Run arbitrary commands against the package manager command and wait for
    /// std::process::ExitStatus
    ///
    /// # Panics
    /// This fn can panic when the defined [``Commands::cmd``] is not found in
    /// path. This can be avoided by using [``verified::Verified``]
    /// or manually ensuring that the [``Commands::cmd``] is valid.
    fn exec_cmds_status<S: AsRef<str>>(&self, cmds: &[S]) -> std::process::ExitStatus {
        self.cmd()
            .args(cmds.iter().map(AsRef::as_ref))
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
    fn exec_cmds_spawn(&self, cmds: &[String]) -> std::process::Child {
        self.cmd()
            .args(cmds)
            .spawn()
            .expect("command executed without a prior check")
    }
}

/// A representation of a package
///
/// This struct contains package's name and version information (optional).
/// It can be constructed with any type that implements `Into<Cow<sr>>`, for
/// example, `&str` and `String`. `Package::from("python")` or with version,
/// `Package::from("python").with_version("3.10.0")`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Package {
    name: String,
    // Untyped version, might be replaced with a strongly typed one
    version: Option<String>,
}

impl Package {
    /// Create new Package with name and version.
    pub fn new(name: &str, version: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            version: version.map(|x| x.to_string()),
        }
    }

    /// Parse a string into [`Self`]
    pub fn from_str(s: &str) -> Self {
        if let Some((name, version)) = s.split_once('@') {
            Package::new(name, Some(version))
        } else {
            Package::new(s, None)
        }
    }

    /// Package name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get version information if present
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = self.version.as_ref() {
            // might be changed later for a better format
            write!(f, "{} - {}", self.name, v)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

/// Available package manager. This is from cli because I can't use
/// MetaPackageManager as `clap::ValueEnum`.
#[derive(Clone, PartialEq, Debug, clap::ValueEnum, strum::EnumIter, strum::EnumCount)]
pub enum AvailablePackageManager {
    Apt,
    Brew,
    Choco,
    Dnf,
    Yum,
    Zypper,
}

impl AvailablePackageManager {
    /// Return the supported pkg format e.g. deb, rpm etc.
    pub fn supported_pkg_formats(&self) -> Vec<PkgFormat> {
        match self {
            Self::Brew => vec![PkgFormat::Bottle],
            Self::Choco => vec![PkgFormat::Exe, PkgFormat::Msi],
            Self::Apt => vec![PkgFormat::Deb],
            Self::Dnf => vec![PkgFormat::Rpm],
            Self::Yum => vec![PkgFormat::Rpm],
            Self::Zypper => vec![PkgFormat::Rpm],
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

/// Pkg Format.
#[derive(Clone)]
pub enum PkgFormat {
    Bottle,
    Exe,
    Msi,
    Rpm,
    Deb,
}

impl PkgFormat {
    /// File extension of package.
    pub fn file_extention(&self) -> String {
        match self {
            Self::Bottle => "tar.gz",
            Self::Exe => "exe",
            Self::Msi => "msi",
            Self::Rpm => "rpm",
            Self::Deb => "deb",
        }
        .to_string()
    }
}
