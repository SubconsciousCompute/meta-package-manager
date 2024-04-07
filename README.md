# Meta Package Manager

A meta package manager for interfacing with multiple distro/platform-specific
package managers using a single, simple, unified interface. `mpm` is both a
Rust library and a CLI utility.

It is inspired by Python's
[meta-package-manager](https://github.com/kdeldycke/meta-package-manager) which is far
ahead in functionality.

# Command-line Interface

The CLI provides a common interface to execute operations using different
package managers. It automatically detects the package managers available on the
system and picks one of them to perform operations by default (the user can also
target a specific package manager if required).

Run `mpm --help` for more details.

# Library usage

See samples in `examples` folder.
