//! Package manager wrapper implementations
//!
//! The wrappers appear in this module based on which feature flag is enabled.
//! If the module is empty, it means that no package manager feature flag is enabled.

mod apt;
mod brew;
mod choco;
mod dnf;
mod yum;

pub use apt::AdvancedPackageTool;
pub use brew::Homebrew;
pub use choco::Chocolatey;
pub use dnf::DandifiedYUM;
pub use yum::YellowdogUpdaterModified;
