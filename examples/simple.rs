/// use trait
use mpm::PackageManager;
use mpm::{MetaPackageManager, Operation};

fn main() -> anyhow::Result<()> {
    let brew = MetaPackageManager::try_new("brew".parse().unwrap())
        .expect("brew could not be initialised");

    // Important: running any commands through the package manager if it is not in
    // path/not installed will result in a panic. See advanced usage for safely
    // constructing verified instances.
    // single package operation (blocking call)
    brew.install("mypackage".parse()?);
    brew.install("packwithver@1.0.0".parse()?);

    // most methods return `ExitStatus` which can be used to check if
    // the operation was successful
    if brew.update_all().success() {
        println!("All packages updated/upgraded");
    }

    // multi pacakge operation (blocking call)
    brew.exec_op(
        &["mypackage".parse().unwrap(), "packwithver".parse().unwrap()],
        Operation::Uninstall,
    );

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
