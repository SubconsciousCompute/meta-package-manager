use std::ops::Deref;

use crate::PackageManager;
use std::process::{Command, Stdio};

/// Wraps `T` that implements [``PackageManager``] and only constructs an instance
/// if the given package manager is installed / is in path.
pub struct Verified<T> {
    inner: T,
}

impl<T: PackageManager> Verified<T> {
    pub fn new(pm: T) -> Option<Self> {
        is_installed(&pm).then_some(Self { inner: pm })
    }
}

impl<T> Deref for Verified<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Converts `T` that implements [``PackageManager``] into `Box<dyn PackageManager>` and only constructs an instance
/// if the given package manager is installed / is in path.
pub struct DynVerified {
    inner: Box<dyn PackageManager>,
}

impl DynVerified {
    pub fn new<P: PackageManager + 'static>(pm: P) -> Option<Self> {
        is_installed(&pm).then_some(Self {
            inner: Box::new(pm),
        })
    }
}

impl Deref for DynVerified {
    type Target = dyn PackageManager;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}

/// Check if package manager is installed on the system
pub fn is_installed<P: PackageManager + ?Sized>(pm: &P) -> bool {
    Command::new(pm.cmd())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}
