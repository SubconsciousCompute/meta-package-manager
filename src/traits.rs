use crate::{Cmd, Operation, Package, PkgFormat};

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
        let o = self
            .cmd()
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
