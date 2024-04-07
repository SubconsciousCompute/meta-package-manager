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

#[macro_use]
pub mod print;
pub use print::*;

pub mod cli;

#[cfg(test)]
mod tests {

    #[cfg(target_family = "unix")]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(target_family = "windows")]
    use std::os::windows::process::ExitStatusExt;
    use std::{
        fmt::Display,
        process::{Command, ExitStatus, Output},
        str::FromStr,
    };

    use super::{Cmd, PackageManagerCommands};
    use crate::{Package, PackageManager, PkgFormat};

    struct MockCommands;

    impl PackageManagerCommands for MockCommands {
        fn cmd(&self) -> Command {
            Command::new("")
        }
        fn get_cmds(&self, _: crate::Cmd) -> Vec<String> {
            vec!["command".to_string()]
        }
        fn get_flags(&self, _: crate::Cmd) -> Vec<String> {
            vec!["flag".to_string()]
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
        fn supported_pkg_formats(&self) -> Vec<PkgFormat> {
            vec![]
        }
    }

    impl PackageManagerCommands for MockPackageManager {
        fn cmd(&self) -> Command {
            Command::new("")
        }
        fn get_cmds(&self, _: Cmd) -> Vec<String> {
            vec!["".to_string()]
        }
        fn exec_cmds(&self, _: &[String]) -> Output {
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
        assert_eq!(coniter.next(), Some("command".to_string()));
        assert_eq!(coniter.next(), Some("flag".to_string()));
        assert_eq!(coniter.next(), Some("arg".to_string()));
    }

    #[test]
    fn default_pm_package_parsing() {
        let pm = MockPackageManager;
        package_assertions(pm.list_installed().into_iter());
        package_assertions(pm.search("").into_iter());
    }

    fn package_assertions(mut listiter: impl Iterator<Item = Package>) {
        assert_eq!(listiter.next(), Package::from_str("package1").ok());
        assert_eq!(listiter.next(), Package::from_str("package2@1.1.0").ok());
        assert_eq!(listiter.next(), Package::from_str("package3").ok());
        assert_eq!(listiter.next(), None);
    }

    #[test]
    fn package_formatting() {
        assert_eq!(MockPackageManager.pkg_format("foo"), "foo");
        assert_eq!(MockPackageManager.pkg_format("foo@0.1.2"), "foo+0.1.2");
    }
}
