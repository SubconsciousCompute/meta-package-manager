use std::fmt::Display;

use clap::ValueEnum;
use strum::{EnumCount, EnumIter};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    managers::{
        AdvancedPackageTool, Chocolatey, DandifiedYUM, Homebrew, YellowdogUpdaterModified, Zypper,
    },
    verify::{DynVerified, Verify},
};

/// Declarative macro for initializing a package manager based on the cfg
/// predicate
///
/// Takes a package manager instance and a cfg predicate (same as the cfg
/// attribute or macro) and attempts to constructs a [``DynVerified``] instance
/// if the cfg predicate evaluates to true, otherwise returns None
macro_rules! if_cfg {
    ($pm:expr, $($cfg:tt)+) => {
        if cfg!($($cfg)+) {
            $pm.verify_dyn()
        } else {
            None
        }
    };
}

/// The enum lists all the supported package managers in one place
///
/// The same enum is used in the Cli command parser.
/// Any package manager names that are too long should have an alias, which will
/// let the users of the CLI access the package manager without having to write
/// the full name.
///
/// The order of the listing is also important. The order dictates priority
/// during selection of the default package manager.
///
/// Adding support to a new package manager involves creating a new variant of
/// its name and writing the appropriate [``Manager::init``] implementation.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, ValueEnum)]
#[value(rename_all = "lower")]
#[non_exhaustive]
pub enum Manager {
    Brew,
    Choco,
    Apt,
    Dnf,
    Yum,
    Zypper,
}

impl Manager {
    /// Initialize the corresponding package maanger from genpack library into a
    /// [``DynVerified``] type
    pub fn init(&self) -> Option<DynVerified> {
        match self {
            Manager::Brew => {
                if_cfg!(Homebrew, target_family = "unix")
            }

            Manager::Choco => {
                if_cfg!(Chocolatey, target_os = "windows")
            }

            Manager::Apt => {
                if_cfg!(
                    AdvancedPackageTool,
                    any(target_os = "linux", target_os = "android")
                )
            }
            Manager::Dnf => {
                if_cfg!(DandifiedYUM, target_os = "linux")
            }
            Manager::Yum => {
                if_cfg!(YellowdogUpdaterModified::default(), target_os = "linux")
            }
            Manager::Zypper => {
                if_cfg!(Zypper, target_os = "linux")
            }
        }
    }

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

impl Display for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Manager::Brew => Homebrew.fmt(f),
            Manager::Choco => Chocolatey.fmt(f),
            Manager::Apt => AdvancedPackageTool.fmt(f),
            Manager::Dnf => DandifiedYUM.fmt(f),
            Manager::Zypper => Zypper.fmt(f),
            Manager::Yum => YellowdogUpdaterModified::default().fmt(f),
        }
    }
}

/// Pkg Format.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
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
