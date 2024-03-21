//! The `once_option` crate defines a single type, [`struct@OnceOption`],
//! with its constructing helper function, [`OnceOption()`].
//!
//! This crate is `no_std`.
//!
//! [`struct@OnceOption`] represents an optional value. Differently from
//! [`Option`], an empty [`struct@OnceOption`] cannot be re-set to contain a
//! value.
//!
//! Additionally, [`struct@OnceOption`] implements [`Deref`](std::ops::Deref)
//! and [`DerefMut`](std::ops::DerefMut) so that its contents can be
//! accessed without pattern-matching (but implicitly unwrapping).
//!
//! It supports comparisons ([`PartialEq`], [`Eq`], [`Ord`] or
//! [`PartialOrd`]) with other [`struct@OnceOption`] containing the same type,
//! as long as the contained type also implements those traits.
//! Furthermore, it can be used as a hash key if the cotnained type
//! is [`Hash`].
//!
//! It supports being displayed if the contained type is [`Display`](std::fmt::Display),
//! and forwards all the formatting traits (except [`Debug`] and
//! [`Pointer`](std::fmt::Pointer)) to its contained-type.
//!
//! # Rationale
//!
//! The main, but not only, purpose of [`struct@OnceOption`] is to simplify
//! and regulate the dropping of members that have methods consuming
//! the values.
//!
//! As an example, this code will fail to compile:
//!
//! ```compile_fail
//! // Warning: this code does *NOT* compile
//!
//! use std::{thread, time::Duration};
//!
//! struct SomeType {
//!     handle: thread::JoinHandle<u32>,
//! }
//!
//! impl SomeType {
//!     fn new() -> Self {
//!         Self {
//!             handle: thread::spawn(|| {
//!                 thread::sleep(Duration::from_secs(5));
//!                 42
//!             }),
//!         }
//!     }
//! }
//!
//! impl Drop for SomeType {
//!     fn drop(&mut self) {
//!         println!("The answer is {}", self.handle.join());
//!     }
//! }
//! ```
//! The compiler will fail with an error like ([try it!](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=39a859f7169e43de84a01d67ec0ca685)):
//!
//! ```text
//! Compiling playground v0.0.1 (/playground)
//! error[E0507]: cannot move out of `self.handle` which is behind a mutable reference
//!     --> src/lib.rs:22:38
//!      |
//! 22   |         println!("The answer is {}", self.handle.join().unwrap());
//!      |                                      ^^^^^^^^^^^ ------ `self.handle` moved due to this method call
//!      |                                      |
//!      |                                      move occurs because `self.handle` has type `JoinHandle<u32>`, which does not implement the `Copy` trait
//!      |
//! note: `JoinHandle::<T>::join` takes ownership of the receiver `self`, which moves `self.handle`
//!     --> /playground/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/thread/mod.rs:1649:17
//!      |
//! 1649 |     pub fn join(self) -> Result<T> {
//!      |                 ^^^^
//!
//! For more information about this error, try `rustc --explain E0507`.
//! error: could not compile `playground` (lib) due to 1 previous error
//!
//! ```
//!
//! [`struct@OnceOption`]  can be used to fix the issue with minimal changes to the
//! code:
//!
//! ```
//! use once_option::OnceOption;
//! use std::{thread, time::Duration};
//!
//! struct SomeType {
//!     handle: OnceOption<thread::JoinHandle<u32>>,
//! }
//!
//! impl SomeType {
//!     fn new() -> Self {
//!         Self {
//!             handle: thread::spawn(|| {
//!                 thread::sleep(Duration::from_secs(5));
//!                 42
//!             }).into(),
//!         }
//!     }
//!
//!     fn thread_id(&self) -> thread::ThreadId {
//!         self.handle.thread().id()
//!     }
//! }
//!
//! impl Drop for SomeType {
//!     fn drop(&mut self) {
//!         println!("The answer is {}", self.handle.take().join().unwrap());
//!     }
//! }
//! ```
//! # Representation
//! [`struct@OnceOption<T>`] has the same ABI of [`Option<T>`]; this means that
//! [`struct@OnceOption<T>`] has the same size, alignment, and function call ABI as [`Option<T>`].
//!
//! An implication of this, is that all the ABI guarantees that [`Option<T>`] makes
//! (i.e. being transmutable from a T under some conditions), also apply to
//! [`struct@OnceOption<T>`]. For further details, see [the documentation for `Option`](core::option#representation).
//!

// We are no-std (docs and tests require std, though)
#![no_std]

#[cfg(test)]
mod tests;

#[cfg(any(test, doc))]
#[macro_use]
extern crate std;

#[cfg(any(test, doc))]
extern crate alloc;

/// [`struct@OnceOption`] represents an optional value. Differently from [`Option`], an empty [`struct@OnceOption`] cannot be re-set to contain a value.
///
/// Check the [crate level documentation](self) for more details.
#[repr(transparent)]
#[derive(Copy, Clone, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct OnceOption<T> {
    inner: Option<T>,
}

#[inline]
#[allow(non_snake_case)]
/// Builds a new [`struct@OnceOption`] containing (and owning) the specified `value`.
///
/// # Examples
///
/// ```
/// # use once_option::OnceOption;
/// let x: OnceOption<u32> = OnceOption(1912);
/// assert_eq!(x.is_some(), true);
/// ```
#[must_use = "if you intend to immediately drop this value, consider using [`drop`] instead"]
pub const fn OnceOption<T>(value: T) -> OnceOption<T> {
    OnceOption { inner: Some(value) }
}

impl<T> OnceOption<T> {
    /// A constant representing a [`struct@OnceOption`] that doesn't contain any value.
    /// Since a [`struct@OnceOption`] cannot be set to contain a value after it has been
    /// emptied, this constant is provided as a helper, but is of dubious utility.
    ///
    /// # Examples
    ///
    /// ```
    /// # use once_option::OnceOption;
    /// let x: OnceOption<u32> = OnceOption::NONE;
    /// assert_eq!(x.is_some(), false);
    /// ```
    pub const NONE: Self = Self { inner: None };

    /// Returns `true` if the once-option contains a value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use once_option::OnceOption;
    /// let x: OnceOption<u32> = OnceOption(1912);
    /// assert_eq!(x.is_some(), true);
    ///
    /// let x: OnceOption<u32> = OnceOption::NONE;
    /// assert_eq!(x.is_some(), false);
    /// ```
    #[inline]
    #[must_use = "if you intended to assert that this has a value, consider `.expect()` or `.unwrap()` instead"]
    pub const fn is_some(&self) -> bool {
        self.inner.is_some()
    }

    /// Returns `true` if the option is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use once_option::OnceOption;
    /// let x: Option<u32> = Some(2);
    /// assert_eq!(x.is_none(), false);
    ///
    /// let x: Option<u32> = None;
    /// assert_eq!(x.is_none(), true);
    /// ```
    #[inline]
    #[must_use = "if you intended to assert that this doesn't have a value, consider \
                  `.expect_none` instead"]
    pub const fn is_none(&self) -> bool {
        !self.is_some()
    }

    /// Returns the contained value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is empty with a custom panic message provided by
    /// `msg`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use once_option::OnceOption;
    /// let x = OnceOption("value");
    /// assert_eq!(x.expect("fruits are healthy"), "value");
    /// ```
    ///
    /// ```should_panic
    /// # use once_option::OnceOption;
    /// let x = OnceOption::<&str>::NONE;
    /// x.expect("fruits are healthy"); // panics with `fruits are healthy`
    /// ```
    #[inline]
    #[track_caller]
    pub fn expect(self, msg: &str) -> T {
        self.inner.expect(msg)
    }

    /// Panics if the [`struct@OnceOption`] is not empty and does contain a value.
    ///
    /// # Panics
    ///
    /// Panics if the value is *not* empty, with a custom panic message provided by
    /// `msg`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use once_option::OnceOption;
    /// let x = OnceOption("value");
    /// assert_eq!(x.expect("fruits are healthy"), "value");
    /// ```
    ///
    /// ```should_panic
    /// # use once_option::OnceOption;
    /// let x = OnceOption::<&str>("something something");
    /// x.expect_none("fruits are healthy"); // panics with `fruits are healthy`
    /// ```
    #[inline]
    #[track_caller]
    pub fn expect_none(self, msg: &str) {
        if self.inner.is_some() {
            panic!("{}", msg);
        }
    }

    /// Takes the value out of the [`struct@OnceOption`], leaving it empty. If the
    /// [`struct@OnceOption`] is already empty, this function will panic.
    ///
    /// Note that this operation cannot be reversed: once a [`struct@OnceOption`] becomes
    /// empty, it cannot get a value again.
    ///
    /// # Examples
    ///
    /// ```
    /// # use once_option::OnceOption;
    /// let mut x = OnceOption(2);
    /// let y = x.take();
    /// assert!(x.is_none());
    /// assert_eq!(y, 2);
    /// ```
    ///
    /// ```should_panic
    /// # use once_option::OnceOption;
    /// let mut x = OnceOption(2);
    /// let y = x.take();
    /// let w = x.take(); // this panics!
    /// ```
    #[inline]
    #[track_caller]
    pub fn take(&mut self) -> T {
        match self.inner.take() {
            Some(value) => value,
            None => Self::fail(),
        }
    }

    /// Returns the contained value, consuming the `self` value. If the
    /// [`struct@OnceOption`] is empty, this function will panic.
    ///
    /// # Examples
    ///
    /// ```
    /// # use once_option::OnceOption;
    /// let mut x = OnceOption(2);
    /// assert_eq!(*x, 2);
    /// x.replace(5);
    /// assert_eq!(*x, 5);
    /// ```
    ///
    /// ```should_panic
    /// # use once_option::OnceOption;
    /// let mut x: OnceOption<u32> = OnceOption::NONE;
    /// x.unwrap(); // this panics!
    /// ```
    #[inline]
    #[track_caller]
    pub fn unwrap(self) -> T {
        match self.inner {
            Some(value) => value,
            None => Self::fail(),
        }
    }

    /// Replaces the actual value in the option by the value given in parameter,
    /// returning the old value if present, or panic'ing otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use once_option::OnceOption;
    /// let mut x = OnceOption(2);
    /// let old = x.replace(5);
    /// assert_eq!(x.take(), 5);
    /// assert_eq!(old, 2);
    /// ```
    ///
    /// ```should_panic
    /// # use once_option::OnceOption;
    /// let mut x: OnceOption<i32> = OnceOption::NONE;
    /// let _ = x.replace(3); // this panics
    /// ```
    #[inline]
    pub fn replace(&mut self, value: T) -> T {
        match self.inner.replace(value) {
            None => Self::fail(),
            Some(v) => v,
        }
    }

    // Default failure method
    fn fail() -> ! {
        panic!(
            "Value has been accessed on an empty once_option::OnceOption<{}>",
            core::any::type_name::<T>()
        )
    }
}

impl<T> Default for OnceOption<T> {
    /// Returns an empty [`struct@OnceOption`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use once_option::OnceOption;
    /// let opt: OnceOption<u32> = OnceOption::default();
    /// assert!(opt.is_none());
    /// ```
    #[inline]
    fn default() -> OnceOption<T> {
        Self::NONE
    }
}

impl<T> From<T> for OnceOption<T> {
    /// Moves `val` into a new [`struct@OnceOption`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use once_option::OnceOption;
    /// let o: OnceOption<u8> = OnceOption::from(67);
    ///
    /// assert_eq!(67, *o);
    /// ```
    /// Alternatively:
    /// ```
    /// # use once_option::OnceOption;
    /// let o: OnceOption<u8> = 67.into();
    ///
    /// assert_eq!(67, *o);
    /// ```
    #[inline]
    fn from(val: T) -> OnceOption<T> {
        OnceOption(val)
    }
}

impl<T> From<Option<T>> for OnceOption<T> {
    /// Moves an option `val` into a new [`struct@OnceOption`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use once_option::OnceOption;
    /// let o: OnceOption<u8> = OnceOption::from(Some(67));
    ///
    /// assert_eq!(67, *o);
    /// ```
    #[inline]
    fn from(val: Option<T>) -> OnceOption<T> {
        Self { inner: val }
    }
}

impl<T> core::ops::Deref for OnceOption<T> {
    type Target = T;

    fn deref(&self) -> &T {
        match self.inner.as_ref() {
            None => Self::fail(),
            Some(r) => r,
        }
    }
}

impl<T> core::ops::DerefMut for OnceOption<T> {
    fn deref_mut(&mut self) -> &mut T {
        match self.inner.as_mut() {
            None => Self::fail(),
            Some(r) => r,
        }
    }
}

macro_rules! impl_formatting_trait {
    ($TRAIT:ident) => {
        impl<T: core::fmt::$TRAIT> core::fmt::$TRAIT for OnceOption<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match &self.inner {
                    None => Self::fail(),
                    Some(v) => core::fmt::$TRAIT::fmt(v, f),
                }
            }
        }
    };
}

impl_formatting_trait!(Display);
impl_formatting_trait!(UpperHex);
impl_formatting_trait!(LowerHex);
impl_formatting_trait!(Octal);
impl_formatting_trait!(Binary);
impl_formatting_trait!(LowerExp);
impl_formatting_trait!(UpperExp);

impl<T: core::fmt::Debug> core::fmt::Debug for OnceOption<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.inner {
            None => f.write_str("OnceOption::NONE"),
            Some(v) => {
                f.write_str("OnceOption(")?;
                core::fmt::Debug::fmt(v, f)?;
                f.write_str(")")
            }
        }
    }
}
