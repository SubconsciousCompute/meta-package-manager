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
mod libtests;
