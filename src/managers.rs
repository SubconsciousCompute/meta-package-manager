#[cfg(feature = "apt")]
mod apt;
#[cfg(feature = "brew")]
mod brew;
#[cfg(feature = "choco")]
mod choco;
#[cfg(feature = "dnf")]
mod dnf;
#[cfg(feature = "yum")]
mod yum;

#[cfg(feature = "apt")]
pub use apt::AdvancedPackageTool;
#[cfg(feature = "brew")]
pub use brew::Homebrew;
#[cfg(feature = "choco")]
pub use choco::Chocolatey;
#[cfg(feature = "dnf")]
pub use dnf::DandifiedYUM;
#[cfg(feature = "yum")]
pub use yum::YellowdogUpdaterModified;
