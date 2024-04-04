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

#[cfg(test)]
mod tests {

    use crate::verify::Verify;
    use crate::PackageManager;

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
        assert!(hb.exec_op(&["hello".into()], Operation::Install).success());
        // list
        assert!(hb.list_installed().iter().any(|p| p.name() == "hello"));
        // update
        assert!(hb.exec_op(&["hello".into()], Operation::Update).success());
        // uninstall
        assert!(hb
            .exec_op(&["hello".into()], Operation::Uninstall)
            .success());
        // TODO: Test AddRepo
    }

    #[cfg(windows)]
    #[test]
    fn test_chocolatey() {
        let choco = crate::managers::Chocolatey;
        let choco = choco.verify().expect("Chocolatey not found in path");
        let pkg = "tac";
        // sync
        assert!(choco.sync().success());
        // search
        assert!(choco.search(pkg).iter().any(|p| p.name() == pkg));
        // install
        assert!(choco.install(pkg.into()).success());
        // list
        assert!(choco.list_installed().iter().any(|p| p.name() == pkg));
        // update
        assert!(choco.update(pkg.into()).success());
        // uninstall
        assert!(choco.uninstall(pkg.into()).success());
        // TODO: Test AddRepo
    }

    // Requires elevated privilages to work
    #[cfg(target_os = "linux")]
    #[test]
    fn test_apt() {
        let apt = crate::managers::AdvancedPackageTool;
        if let Some(apt) = apt.verify() {
            let pkg = "hello";
            // sync
            assert!(apt.sync().success());
            // search
            assert!(apt.search(pkg).iter().any(|p| p.name() == "hello"));
            // install
            assert!(apt.install(pkg.into()).success());
            // list
            assert!(apt.list_installed().iter().any(|p| p.name() == "hello"));
            // update
            assert!(apt.update(pkg.into()).success());
            // uninstall
            assert!(apt.uninstall(pkg.into()).success());
            // TODO: Test AddRepo
        } else {
            eprintln!("apt is not found");
        }
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

    fn dnf_yum_cases(man: impl crate::PackageManager) {
        if let Some(man) = man.verify() {
            let pkg = "hello";
            // sync
            assert!(man.sync().success());
            // search
            assert!(man.search(pkg).iter().any(|p| p.name() == "hello.x86_64"));
            // install
            assert!(man.install(pkg.into()).success());
            // list
            assert!(man
                .list_installed()
                .iter()
                .any(|p| p.name() == "hello.x86_64"));
            // update
            assert!(man.update(pkg.into()).success());
            // uninstall
            assert!(man.uninstall(pkg.into()).success());
            // TODO: Test AddRepo
        } else {
            eprintln!("dnf not found");
        }
    }
}
