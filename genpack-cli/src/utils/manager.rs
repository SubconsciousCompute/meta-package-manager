use std::fmt::Display;

use clap::ValueEnum;
use genpack::{
    managers::{AdvancedPackageTool, Chocolatey, DandifiedYUM, Homebrew, YellowdogUpdaterModified},
    verify::{DynVerified, Verify},
};
use strum::{EnumCount, EnumIter};

/// Declarative macro for initializing a package manager based on the cfg predicate
///
/// Takes a package manager instance and a cfg predicate (same as the cfg attribute or macro) and attempts to constructs a [``DynVerified``] instance
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
/// Any package manager names that are too long should have an alias, which will let the users of the CLI
/// access the package manager without having to write the full name.
///
/// The order of the listing is also important. The order dictates priority during selection of the default
/// package manager.
///
/// Adding support to a new package manager involves creating a new variant of its name and writing the appropriate
/// [``Manager::init``] implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, ValueEnum)]
#[value(rename_all = "lower")]
#[non_exhaustive]
pub enum Manager {
    Brew,
    Choco,
    Apt,
    Dnf,
    Yum,
}

impl Manager {
    /// Initialize the corresponding package maanger from genpack library into a [``DynVerified``] type

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
            Manager::Yum => YellowdogUpdaterModified::default().fmt(f),
        }
    }
}
