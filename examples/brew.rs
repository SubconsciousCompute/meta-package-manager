use mpm::{MetaPackageManager, Operation, PackageManager};

fn main() -> anyhow::Result<()> {
    let brew = MetaPackageManager::new_if_available("brew".parse().unwrap())
        .expect("brew could not be initialised");

    // Important: running any commands through the package manager if it is not in
    // path/not installed will result in a panic. See advanced usage for safely
    // constructing verified instances.
    // single package operation (blocking call)
    brew.install("mypackage", false);
    brew.install("packwithver@1.0.0", false);

    // most methods return `ExitStatus` which can be used to check if
    // the operation was successful
    if brew.update_all(false).success() {
        println!("All packages updated/upgraded");
    }

    // multi pacakge operation (blocking call)
    let mut pkg = "mypackage".into();
    brew.execute_pkg_command(&mut pkg, Operation::Uninstall, false);

    // get packages matching search string
    for p in brew.search("python") {
        println!("{p}");
    }

    // list installed packages
    for p in brew.list_installed() {
        println!("{p}");
    }
    Ok(())
}
