use genpack::verify::Verify;
use genpack::*;

#[cfg(not(target_os = "windows"))]
#[test]
fn homebrew() {
    let hb = managers::HomeBrew;
    let hb = hb.verify().expect("HomeBrew not found in path");
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

#[cfg(target_os = "windows")]
#[test]
fn chocolatey() {
    let choco = managers::Chocolatey;
    let choco = choco.verify().expect("Chocolatey not found in path");
    // sync
    assert!(choco.sync().success());
    // search
    assert!(choco.search("rust").iter().any(|p| p.name() == "rust"));
    // TODO: Test Install, Uninstall, Update, List and AddRepo
}

// Requires elevated privilages to work
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
