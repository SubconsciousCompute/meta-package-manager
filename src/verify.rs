//! This module contains types that are used to signify that a package manager
//! is installed / is in path and is safe to use.
//!
//! The [``crate::Commands``] trait (which the [``PackageManager``] trait relies
//! on) internally uses `unwrap` on [``std::process::Command``] when it is
//! executed to avoid extra error-handling on the user side. This is safe and
//! won't cause a panic when the primary command of the given package maanger
//! works (i.e. is in path), which is the assumption the API makes.
//! However, the user cannot always be trusted to ensure this assumption holds
//! true. Therefore, one can rely on the functionality provided by this
//! module to check if a package manager is installed / is in path, and then
//! construct a type that marks that it is now safe to use. This is done by
//! leveraging the type system: the verified types can only be constructed using
//! their provided constructor functions. They internally use the
//! [``is_installed``] function and return `Some(Self)` only if it returns
//! `true`.
//!
//! The type [``Verified``] is a generic wrapper that works with any `T` that
//! implements the [``PackageManager``] trait. [``DynVerified``] is similar,
//! except that it internally stores the type `T` as `Box<dyn PackageManager>`
//! to allow dynamic dispatch with ease. These types can be constructed manaully
//! by using their respective constructors or use the [``Verify``] trait, which
//! provides a blanket implementation and lets you construct them directly from
//! any instance of a type that implements [``PackageManager``]. For example,
//! `MyPackageMan::new().verify()` or `MyPackageMan::new().verify_dyn()`.
use std::{fmt::Display, ops::Deref, process::Stdio};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::PackageManager;

/// Wraps `T` that implements [``PackageManager``] and only constructs an
/// instance if the given package manager is installed / is in path.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug)]
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

impl<T: PackageManager> Display for Verified<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.inner))
    }
}

/// Converts `T` that implements [``PackageManager``] into `Box<dyn
/// PackageManager>` and only constructs an instance if the given package
/// manager is installed / is in path.
#[derive(Debug)]
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

impl Display for DynVerified {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.inner))
    }
}

/// Check if package manager is installed on the system
///
/// The current implementation merely runs the primary package-manager command
/// and checks if it returns an error or not.
pub fn is_installed<P: PackageManager + ?Sized>(pm: &P) -> bool {
    tracing::trace!("Checking if {pm:?} is installed: {:?}", pm.cmd());
    pm.cmd()
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .is_ok()
}

/// Helper trait that lets you construct a verified package manager instance
/// that is known to be installed or in path, and is safe to be interacted with.
///
/// This trait has a blanket implementation for all T that implement
/// PackageManager
pub trait Verify: PackageManager
where
    Self: Sized,
{
    /// Creates an instance of [``Verified``], which signifies that the package
    /// manager is installed and is safe to be interacted with.
    fn verify(self) -> Option<Verified<Self>> {
        Verified::new(self)
    }

    /// Creates an instance of [``DynVerified``], which signifies that the
    /// package manager is installed and is safe to be interacted with.
    ///
    /// Note: This internally converts and stores `Self` as `dyn PackageManager`
    fn verify_dyn(self) -> Option<DynVerified>
    where
        Self: 'static,
    {
        DynVerified::new(self)
    }
}

impl<T> Verify for T where T: PackageManager {}
