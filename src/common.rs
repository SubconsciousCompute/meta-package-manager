//! Common types and traits.

use std::{borrow::Cow, fmt::Display};

use anyhow::Context;

/// Primary interface for implementing a package manager
///
/// Multiple package managers can be grouped together as dyn PackageManager.
#[ambassador::delegatable_trait]
pub trait PackageManager: PackageManagerCommands + std::fmt::Debug + std::fmt::Display {
    /// Defines a delimeter to use while formatting package name and version
    ///
    /// For example, HomeBrew supports `<name>@<version>` and APT supports
    /// `<name>=<version>`. Their appropriate delimiters would be '@' and
    /// '=', respectively. For package managers that require additional
    /// formatting, overriding the default trait methods would be the way to go.
    fn pkg_delimiter(&self) -> char;

    /// Return the list of supported package extensions.
    fn supported_pkg_formats(&self) -> Vec<PkgFormat>;

    /// Get a formatted string of the package that can be passed into package
    /// manager's cli.
    ///
    /// If package URL is set, the url is passed to cli. Note that not all
    /// package manager supports installing using url. We override this
    /// function.
    fn reformat_for_command(&self, pkg: &mut Package) -> String {
        pkg.cli_display(self.pkg_delimiter())
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
    fn search(&self, query: &str) -> Vec<Package> {
        let cmds = self.consolidated(Cmd::Search, None, &[query.to_string()]);
        let out = self.exec_cmds(&cmds);
        self.parse_output(&out.stdout)
    }

    /// Sync package manaager repositories
    fn sync(&self) -> std::process::ExitStatus {
        self.exec_cmds_status(&self.consolidated::<&str>(Cmd::Sync, None, &[]))
    }

    /// Update/upgrade all packages
    fn update_all(&self) -> std::process::ExitStatus {
        self.exec_cmds_status(&self.consolidated::<&str>(Cmd::UpdateAll, None, &[]))
    }

    /// Install a single package
    ///
    /// For multi-package operations, see
    /// [``PackageManager::execute_pkg_command``]
    fn install<P: Into<Package> + Clone + std::fmt::Debug>(
        &self,
        pkg: P,
    ) -> std::process::ExitStatus {
        let mut pkg = pkg.into();
        self.execute_pkg_command(&mut pkg, Operation::Install)
    }

    /// Uninstall a single package
    ///
    /// For multi-package operations, see
    /// [``PackageManager::execute_pkg_command``]
    fn uninstall<P: Into<Package> + Clone + std::fmt::Debug>(
        &self,
        pkg: P,
    ) -> std::process::ExitStatus {
        let mut pkg = pkg.into();
        self.execute_pkg_command(&mut pkg, Operation::Uninstall)
    }

    /// Update a single package
    ///
    /// For multi-package operations, see
    /// [``PackageManager::execute_pkg_command``]
    fn update<P: Into<Package> + Clone + std::fmt::Debug>(
        &self,
        pkg: P,
    ) -> std::process::ExitStatus {
        let mut pkg = pkg.into();
        self.execute_pkg_command(&mut pkg, Operation::Update)
    }

    /// List installed packages
    fn list_installed(&self) -> Vec<Package> {
        let out = self.exec_cmds(&self.consolidated::<&str>(Cmd::List, None, &[]));
        self.parse_output(&out.stdout)
    }

    /// Execute package manager command.
    fn execute_pkg_command(&self, pkg: &mut Package, op: Operation) -> std::process::ExitStatus {
        tracing::debug!("> Operation {op:?} on {pkg:?}...");
        let command = match op {
            Operation::Install => Cmd::Install,
            Operation::Uninstall => Cmd::Uninstall,
            Operation::Update => Cmd::Update,
        };

        let fmt = self.reformat_for_command(pkg);
        tracing::debug!(">> {pkg:?} -> {fmt}");

        let cmds = self.consolidated(command, Some(pkg), &[fmt.clone()]);
        tracing::debug!(">> {pkg} -> {fmt} -> {cmds:?}");
        self.exec_cmds_status(&cmds)
    }

    /// Add third-party repository to the package manager's repository list
    ///
    /// Since the implementation might greatly vary among different package
    /// managers this method returns a `Result` instead of the usual
    /// `std::process::ExitStatus`.
    fn add_repo(&self, repo: &str) -> anyhow::Result<()> {
        let cmds = self.consolidated(Cmd::AddRepo, None, &[repo.to_string()]);
        let s = self.exec_cmds_status(&cmds);
        anyhow::ensure!(s.success(), "Error adding repo");
        Ok(())
    }
}

/// Representation of a package manager command
///
/// All the variants are the type of commands that a type that imlements
/// [``PackageManagerCommands``] and [``PackageManager``] (should) support.
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
/// Only [``PackageManagerCommands::cmd``] and [``Commands::commands``] are
/// required, the rest are simply conviniece methods that internally call
/// [``PackageManagerCommands::commands``]. The trait [``PackageManager``]
/// depends on this to provide default implementations.
#[ambassador::delegatable_trait]
pub trait PackageManagerCommands {
    /// Primary command of the package manager. For example, 'brew', 'apt', and
    /// 'dnf', constructed with [``std::process::Command::new``].
    fn cmd(&self) -> std::process::Command;

    /// Returns the appropriate command/s for the given supported command type.
    /// Check [``crate::common::Cmd``] enum to see all supported commands.
    fn get_cmds(&self, cmd: Cmd, pkg: Option<&Package>) -> Vec<String>;

    /// Returns the appropriate flags for the given command type. Check
    /// [``crate::common::Cmd``] enum to see all supported commands.
    ///
    /// Flags are optional, which is why the default implementation returns an
    /// empty slice
    fn get_flags(&self, _cmd: Cmd) -> Vec<String> {
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
    fn consolidated<S: AsRef<str>>(
        &self,
        cmd: Cmd,
        pkg: Option<&Package>,
        args: &[S],
    ) -> Vec<String> {
        let mut commands = self.get_cmds(cmd, pkg);
        commands.append(&mut self.get_flags(cmd));
        commands.append(&mut args.iter().map(|x| x.as_ref().to_string()).collect());
        commands
    }

    /// Run arbitrary commands against the package manager command and get
    /// output
    ///
    /// # Panics
    /// This fn can panic when the defined [``PackageManagerCommands::cmd``] is
    /// not found in path. This can be avoided by using
    /// [``verified::Verified``] or manually ensuring that the
    /// [``PackageManagerCommands::cmd``] is valid.
    fn exec_cmds(&self, cmds: &[String]) -> std::process::Output {
        self.ensure_sudo();
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
    /// This fn can panic when the defined [``PackageManagerCommands::cmd``] is
    /// not found in path. This can be avoided by using
    /// [``verified::Verified``] or manually ensuring that the
    /// [``PackageManagerCommands::cmd``] is valid.
    fn exec_cmds_status<S: AsRef<str> + std::fmt::Debug>(
        &self,
        cmds: &[S],
    ) -> std::process::ExitStatus {
        self.ensure_sudo();
        tracing::info!("Executing {:?} with args {:?}", self.cmd(), cmds);
        let o = self.cmd()
            .args(cmds.iter().map(AsRef::as_ref))
            .output()
            .expect("command executed without a prior check");
        tracing::debug!(">>> {o:?}");
        o.status
    }

    /// Run arbitrary commands against the package manager command and return
    /// handle to the spawned process
    ///
    /// # Panics
    /// This fn can panic when the defined [``PackageManagerCommands::cmd``] is
    /// not found in path. This can be avoided by using
    /// [``verified::Verified``] or manually ensuring that the
    /// [``PackageManagerCommands::cmd``] is valid.
    fn exec_cmds_spawn(&self, cmds: &[String]) -> std::process::Child {
        self.ensure_sudo();
        tracing::info!("Executing {:?} with args {:?}", self.cmd(), cmds);
        self.cmd()
            .args(cmds)
            .spawn()
            .expect("command executed without a prior check")
    }

    /// Ensure that we are in sudo mode.
    fn ensure_sudo(&self) {
        #[cfg(target_os = "linux")]
        if let Err(e) = sudo::with_env(&["CARGO_", "MPM_LOG", "RUST_LOG"]) {
            tracing::warn!("Failed to elevate to sudo: {e}.");
        }
    }

    /// Check is package manager is available.
    fn is_available(&self) -> bool {
        match self.cmd().arg("--version").output() {
            Err(_) => false,
            Ok(output) => output.status.success(),
        }
    }
}

/// A representation of a package
///
/// This struct contains package's name and version information (optional).
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize)]
pub struct Package {
    /// name of the package
    name: String,

    /// Untyped version, might be replaced with a strongly typed one
    version: Option<String>,

    /// Url of this package. A local package can be passed as "file://" URI.
    url: Option<url::Url>,
}

impl Package {
    /// Create new Package with name and version.
    pub fn new(name: &str, version: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            version: version.map(|v| v.to_string()),
            url: None,
        }
    }

    /// Name of the package
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Package name for cli.
    pub fn cli_display(&self, pkg_delimiter: char) -> String {
        if let Some(url) = self.url() {
            if url.scheme() == "file" {
                tracing::debug!("Found on-disk path. Stripping file://...");
                return url.as_str().strip_prefix("file://").unwrap().to_string();
            }
            return url.as_str().to_string();
        }
        if let Some(version) = &self.version {
            return format!("{}{}{}", self.name, pkg_delimiter, version);
        }
        self.name.clone()
    }

    /// Get version information if present
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }

    /// Get version information if present
    pub fn url(&self) -> Option<&url::Url> {
        self.url.as_ref()
    }

    /// Turn remote url to local file based URI
    pub fn make_available_on_disk(
        &mut self,
        output: Option<&std::path::Path>,
        force: bool,
    ) -> anyhow::Result<std::path::PathBuf> {
        use std::io::Write;

        anyhow::ensure!(
            self.url().is_some(),
            "There is no URL associated with this package"
        );
        let url = self.url().context("missing URL")?;
        anyhow::ensure!(
            url.scheme() != "file",
            "Package already points to local file {url:?}"
        );

        let pkgpath = match output {
            Some(p) => p.into(),
            None => std::env::temp_dir().join(
                url.path_segments()
                    .context("missing path in url")?
                    .last()
                    .context("missing filepath in url")?,
            ),
        };

        // download to disk.
        if !pkgpath.exists() || force {
            tracing::debug!("Downloading package from `{url}` (force={force})...");
            let resp = reqwest::blocking::Client::builder()
                .timeout(None)
                .build()?
                .get(url.as_str())
                .send()?;

            let bytes = resp.bytes()?;
            tracing::debug!(" ... fetched {} MB.", bytes.len() / 1024 / 1024);

            let mut buffer = std::fs::File::create(&pkgpath)?;
            buffer.write_all(&bytes)?;
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        anyhow::ensure!(pkgpath.is_file(), "Failed to download {url} -> {pkgpath:?}");
        self.url = format!("file://{}", pkgpath.display()).parse().ok();
        Ok(pkgpath)
    }
}

impl std::str::FromStr for Package {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        if let Ok(url) = url::Url::parse(s) {
            let name = url
                .path_segments()
                .context("can not determine pane from the url")?
                .last()
                .expect("can't determine package name from the url");
            let mut fragments = std::collections::HashMap::new();
            for frag in url.fragment().unwrap_or("").split(',') {
                let mut fs = frag.splitn(2, '=');
                if let Some(key) = fs.next() {
                    if let Some(value) = fs.next() {
                        fragments.insert(key, value.to_string());
                    }
                }
            }
            return Ok(Self {
                name: name.to_string(),
                version: fragments.remove("version"),
                url: Some(url),
            });
        }

        if let Some((name, version)) = s.split_once('@') {
            Ok(Package::new(name, Some(version)))
        } else {
            Ok(Package::new(s, None))
        }
    }
}

impl std::convert::From<&str> for Package {
    fn from(s: &str) -> Self {
        s.parse().expect("invalid format")
    }
}

impl std::convert::From<&std::path::Path> for Package {
    fn from(p: &std::path::Path) -> Self {
        let s = format!("file://{}", p.display());
        s.parse().expect("invalid format")
    }
}

impl Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(v) = self.version() {
            // might be changed later for a better format
            write!(f, "{} - {}", self.name, v)
        } else {
            write!(f, "{}", self.name)
        }
    }
}

impl tabled::Tabled for Package {
    const LENGTH: usize = 40;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.name.clone().into(),
            self.version.as_deref().unwrap_or("~").into(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec!["name".into(), "version".into()]
    }
}

/// Available package manager. This is from cli because I can't use
/// MetaPackageManager as `clap::ValueEnum`.
#[derive(
    Clone,
    PartialEq,
    Debug,
    clap::ValueEnum,
    strum::Display,
    strum::EnumIter,
    strum::EnumCount,
    strum::EnumString,
)]
#[strum(ascii_case_insensitive)]
pub enum AvailablePackageManager {
    Apt,
    Brew,
    Choco,
    Dnf,
    Yum,
    Zypper,
}

/// Operation type to execute using [``Package::execute_pkg_command``]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Operation {
    Install,
    Uninstall,
    Update,
}

/// Pkg Format.
#[derive(Clone, PartialEq, Eq, Hash)]
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
