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
}
