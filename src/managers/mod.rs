//! Package manager wrapper implementations
//!
//! The wrappers appear in this module based on which feature flag is enabled.
//! If the module is empty, it means that no package manager feature flag is
//! enabled.

pub mod apt;
pub mod brew;
pub mod choco;
pub mod dnf;
pub mod yum;
pub mod zypper;

pub use apt::AdvancedPackageTool;
pub use brew::Homebrew;
pub use choco::Chocolatey;
pub use dnf::DandifiedYUM;
pub use yum::YellowdogUpdaterModified;
pub use zypper::Zypper;
