//! Package manager wrapper implementations
//!
//! The wrappers appear in this module based on which feature flag is enabled.
//! If the module is empty, it means that no package manager feature flag is
//! enabled.

use ambassador::Delegate;
use anyhow::Context;
use strum::IntoEnumIterator;

pub mod apt;
pub mod brew;
pub mod choco;
pub mod dnf;
pub mod yum;
pub mod zypper;

use apt::AdvancedPackageTool;
use brew::Homebrew;
use choco::Chocolatey;
use dnf::DandifiedYUM;
use yum::YellowdogUpdaterModified;
use zypper::Zypper;

use crate::common::*;

/// Enum of all supported package managers.
#[derive(Debug, Delegate, strum::EnumIter, strum::EnumCount)]
#[delegate(crate::PackageManagerCommands)]
#[delegate(crate::PackageManager)]
pub enum MetaPackageManager {
    Apt(AdvancedPackageTool),
    Brew(Homebrew),
    Choco(Chocolatey),
    Dnf(DandifiedYUM),
    Yum(YellowdogUpdaterModified),
    Zypper(Zypper),
}

impl MetaPackageManager {
    /// Construct a new `MetaPackageManager` from a given package manager.
    pub fn new(manager: AvailablePackageManager) -> Self {
        tracing::debug!("Creating meta-package-manager interface for {manager:?}");
        match manager {
            AvailablePackageManager::Apt => Self::Apt(AdvancedPackageTool),
            AvailablePackageManager::Brew => Self::Brew(Homebrew),
            AvailablePackageManager::Choco => Self::Choco(Chocolatey),
            AvailablePackageManager::Dnf => Self::Dnf(DandifiedYUM),
            AvailablePackageManager::Yum => Self::Yum(YellowdogUpdaterModified::default()),
            AvailablePackageManager::Zypper => Self::Zypper(Zypper),
        }
    }

    /// Construct a new `MetaPackageManager` from a given package manager but
    /// make sure that it exists on this system.
    pub fn new_if_available(manager: AvailablePackageManager) -> anyhow::Result<Self> {
        let mpm = Self::new(manager);
        if !mpm.is_available() {
            anyhow::bail!("failed to run {mpm} command")
        }
        Ok(mpm)
    }

    /// Try to find the system package manager.
    ///
    /// First enum variant is given the highest priority, second, the second
    /// highest, and so on.
    pub fn new_default() -> anyhow::Result<Self> {
        AvailablePackageManager::iter()
            .find_map(|m| Self::new_if_available(m).ok())
            .context("no supported package manager found")
    }
}

impl std::fmt::Display for MetaPackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetaPackageManager::Brew(_) => Homebrew.fmt(f),
            MetaPackageManager::Choco(_) => Chocolatey.fmt(f),
            MetaPackageManager::Apt(_) => AdvancedPackageTool.fmt(f),
            MetaPackageManager::Dnf(_) => DandifiedYUM.fmt(f),
            MetaPackageManager::Zypper(_) => Zypper.fmt(f),
            MetaPackageManager::Yum(_) => YellowdogUpdaterModified::default().fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    fn test_supported_fmts() {
        let mpm = MetaPackageManager::new_default().unwrap();
        let exts = mpm.supported_pkg_formats();
        assert!(!exts.is_empty());
    }

    #[test]
    #[traced_test]
    #[cfg(target_os = "linux")]
    fn test_url_support() {
        let mpm = MetaPackageManager::new_default().unwrap();
        let exts = mpm.supported_pkg_formats();
        let url = if exts.contains(&PkgFormat::Deb) {
            "https://www.clamav.net/downloads/production/clamav-1.3.0.linux.x86_64.deb"
        } else if exts.contains(&PkgFormat::Rpm) {
            "https://www.clamav.net/downloads/production/clamav-1.3.0.linux.x86_64.rpm"
        } else {
            eprintln!("Only deb/rpm are supported");
            return;
        };
        let s = mpm.install(url);
        assert!(s.success());
    }
}
