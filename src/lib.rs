//! meta-package-manager (mpm) library.
//!
//! 1. initialize a package manager
//!
//! ```ignore
//! let pkg_manager = MetaPackageManager::try_default().unwrap();
//! println!("{}", pkg_manager.about());
//! ```
//!
//! or
//!
//! ```ignore
//! let pkg_manager = MetaPackageManager::new("choco").unwrap();
//! println!("{}", pkg_manager.about());
//! ```
//!
//! 2. Install a package with optional version
//!
//! ```ignore
//! pkg_manager.install("firefox", Some("101")).uwnrap();
//! ```
//!
//! 3. Search a package
//!
//! ```ignore
//! pkg_manager.search("firefox").uwnrap();
//! ```

#[macro_use]
pub mod common;
pub use common::*;

pub mod managers;
pub use managers::*;

#[cfg(feature = "cli")]
pub mod utils;

#[cfg(test)]
mod tests {

    #[cfg(target_family = "unix")]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(target_family = "windows")]
    use std::os::windows::process::ExitStatusExt;
    use std::{
        fmt::Display,
        process::{Command, ExitStatus, Output},
    };

    use super::{Cmd, Commands};
    use crate::{Package, PackageManager};

    struct MockCommands;

    impl Commands for MockCommands {
        fn cmd(&self) -> Command {
            Command::new("")
        }
        fn get_cmds(&self, _: crate::Cmd) -> &'static [&'static str] {
            &["command"]
        }
        fn get_flags(&self, _: crate::Cmd) -> &'static [&'static str] {
            &["flag"]
        }
    }

    #[derive(Debug)]
    struct MockPackageManager;

    impl Display for MockPackageManager {
        fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            unimplemented!()
        }
    }

    impl PackageManager for MockPackageManager {
        fn pkg_delimiter(&self) -> char {
            '+'
        }
    }

    impl Commands for MockPackageManager {
        fn cmd(&self) -> Command {
            Command::new("")
        }
        fn get_cmds(&self, _: Cmd) -> &'static [&'static str] {
            &[""]
        }
        fn exec_cmds(&self, _: &[&str]) -> Output {
            let out = br#"
            package1
            package2+1.1.0
            package3   
        "#;
            Output {
                status: ExitStatus::from_raw(0),
                stdout: out.to_vec(),
                stderr: vec![],
            }
        }
    }

    #[test]
    fn default_cmd_consolidated_order() {
        let mock = MockCommands;
        let con = mock.consolidated(Cmd::Install, &["arg"]);
        let mut coniter = con.into_iter();
        assert_eq!(coniter.next(), Some("command"));
        assert_eq!(coniter.next(), Some("arg"));
        assert_eq!(coniter.next(), Some("flag"));
    }

    #[test]
    fn default_pm_package_parsing() {
        let pm = MockPackageManager;
        package_assertions(pm.list_installed().into_iter());
        package_assertions(pm.search("").into_iter());
    }

    fn package_assertions<'a>(mut listiter: impl Iterator<Item = Package<'a>>) {
        assert_eq!(listiter.next(), Some(Package::from("package1")));
        assert_eq!(
            listiter.next(),
            Some(Package::from("package2").with_version("1.1.0"))
        );
        assert_eq!(listiter.next(), Some(Package::from("package3")));
        assert_eq!(listiter.next(), None);
    }

    #[test]
    fn package_formatting() {
        let pkg = Package::from("package");
        assert_eq!(MockPackageManager.pkg_format(&pkg), "package");
        let pkg = pkg.with_version("0.1.0");
        assert_eq!(MockPackageManager.pkg_format(&pkg), "package+0.1.0");
    }

    #[test]
    fn package_version() {
        let pkg = Package::from("test");
        assert!(!pkg.has_version());
        let pkg = pkg.with_version("1.1");
        assert!(pkg.has_version());
    }

    #[test]
    fn package_version_replace() {
        let pkg = Package::from("test").with_version("1.0");
        assert_eq!(pkg.version(), Some("1.0"));
        let pkg = pkg.with_version("2.0");
        assert_eq!(pkg.version(), Some("2.0"));
    }
}
