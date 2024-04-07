# Meta Package Manager

A meta package manager for interfacing with multiple distro/platform-specific
package managers using a single, simple, unified interface. `mpm` is both a
Rust library and a CLI utility.

It is inspired by Python's
[meta-package-manager](https://github.com/kdeldycke/meta-package-manager) which is far
ahead in functionality.

## Basic Usage

```rust
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
    brew.execute_pkg_command(
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
```

# Command-line Interface

The CLI provides a common interface to execute operations using different
package managers. It automatically detects the package managers available on the
system and picks one of them to perform operations by default (the user can also
target a specific package manager if required).

Run `mpm --help` for more details.
