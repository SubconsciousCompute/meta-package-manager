//! Tests

use mpm::verify::Verify;
use mpm::*;

#[cfg(not(target_os = "osx"))]
#[test]
fn homebrew() {
    let hb = managers::Homebrew;
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
fn chocolatey() {
    let choco = managers::Chocolatey;
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
#[cfg(feature = "apt")]
#[cfg(target_os = "linux")]
#[ignore]
#[test]
fn apt() {
    let apt = managers::AdvancedPackageTool;
    let apt = apt.verify().expect("Apt not found in path");
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
}

// Requires elevated privilages to work
#[cfg(target_os = "linux")]
#[ignore]
#[test]
fn dnf() {
    dnf_yum_cases(managers::DandifiedYUM)
}

// Requires elevated privilages to work
#[cfg(target_os = "linux")]
#[ignore]
#[test]
fn yum() {
    dnf_yum_cases(managers::YellowdogUpdaterModified::default())
}

fn dnf_yum_cases(man: impl PackageManager) {
    let man = man.verify().expect("Dnf not found in path");
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
}
