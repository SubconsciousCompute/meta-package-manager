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
    pub fn try_new(manager: AvailablePackageManager) -> anyhow::Result<Self> {
        tracing::info!("Creating meta package manager interface for {manager:?}");
        let mpm = match manager {
            AvailablePackageManager::Apt => Self::Apt(AdvancedPackageTool),
            AvailablePackageManager::Brew => Self::Brew(Homebrew),
            AvailablePackageManager::Choco => Self::Choco(Chocolatey),
            AvailablePackageManager::Dnf => Self::Dnf(DandifiedYUM),
            AvailablePackageManager::Yum => Self::Yum(YellowdogUpdaterModified::default()),
            AvailablePackageManager::Zypper => Self::Zypper(Zypper),
        };

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
            .find_map(|m| Self::try_new(m).ok())
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

    use crate::{PackageManager, PackageManagerCommands};

    #[cfg(target_os = "osx")]
    #[test]
    fn test_homebrew() {
        let hb = crate::managers::Homebrew;
        let hb = hb.verify().expect("Homebrew not found in path");
        // sync
        assert!(hb.sync().success());
        // search
        assert!(hb.search("hello").iter().any(|p| p.name() == "hello"));
        // install
        assert!(hb
            .exec_op(&["hello".parse().unwrap()], Operation::Install)
            .success());
        // list
        assert!(hb.list_installed().iter().any(|p| p.name() == "hello"));
        // update
        assert!(hb
            .exec_op(&["hello".parse().unwrap()], Operation::Update)
            .success());
        // uninstall
        assert!(hb
            .exec_op(&["hello".parse().unwrap()], Operation::Uninstall)
            .success());
        // TODO: Test AddRepo
    }

    #[cfg(windows)]
    #[test]
    fn test_chocolatey() {
        let choco = crate::managers::Chocolatey;
        let pkg = "tac";
        // sync
        assert!(choco.sync().success());
        // search
        assert!(choco.search(pkg).iter().any(|p| p.name() == pkg));
        // install
        assert!(choco.install(pkg.parse().unwrap()).success());
        // list
        assert!(choco.list_installed().iter().any(|p| p.name() == pkg));
        // update
        assert!(choco.update(pkg.parse().unwrap()).success());
        // uninstall
        assert!(choco.uninstall(pkg.parse().unwrap()).success());
        // TODO: Test AddRepo
    }

    // Requires elevated privilages to work
    #[cfg(target_os = "linux")]
    #[test]
    fn test_apt() {
        let apt = crate::managers::AdvancedPackageTool;
        if !apt.is_available() {
            println!("apt is not available");
            return;
        }

        let pkg = "hello";
        // sync
        assert!(apt.sync().success());
        // search
        assert!(apt.search(pkg).iter().any(|p| p.name() == "hello"));
        // install
        assert!(apt.install(pkg.parse().unwrap()).success());
        // list
        assert!(apt.list_installed().iter().any(|p| p.name() == "hello"));
        // update
        assert!(apt.update(pkg.parse().unwrap()).success());
        // uninstall
        assert!(apt.uninstall(pkg.parse().unwrap()).success());
        // TODO: Test AddRepo
    }

    // Requires elevated privilages to work
    #[cfg(target_os = "linux")]
    #[test]
    fn test_dnf() {
        dnf_yum_cases(crate::managers::DandifiedYUM)
    }

    // Requires elevated privilages to work
    #[cfg(target_os = "linux")]
    #[test]
    fn test_yum() {
        dnf_yum_cases(crate::managers::YellowdogUpdaterModified::default())
    }

    #[cfg(target_os = "linux")]
    fn dnf_yum_cases(man: impl crate::PackageManager) {
        if !man.is_available() {
            println!("Yum is not available");
            return;
        }
        let pkg = "hello";
        // sync
        assert!(man.sync().success());
        // search
        assert!(man.search(pkg).iter().any(|p| p.name() == "hello.x86_64"));
        // install
        assert!(man.install(pkg.parse().unwrap()).success());
        // list
        assert!(man
            .list_installed()
            .iter()
            .any(|p| p.name() == "hello.x86_64"));
        // update
        assert!(man.update(pkg.parse().unwrap()).success());
        // uninstall
        assert!(man.uninstall(pkg.parse().unwrap()).success());
        // TODO: Test AddRepo
    }
}
