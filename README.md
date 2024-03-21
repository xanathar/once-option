# once-option

![CI](https://github.com/xanathar/logplotter/actions/workflows/CI.yml/badge.svg) ![Crates.io](https://img.shields.io/crates/v/once-option) ![docs.rs](https://img.shields.io/docsrs/once-option) ![Crates.io](https://img.shields.io/crates/d/once-option) ![Crates.io](https://img.shields.io/crates/l/once-option)

The `once_option` crate defines a single type, `OnceOption`,
with its constructing helper function, `OnceOption()`.

This crate is `no_std`.

`OnceOption` represents an optional value. Differently from
`Option`, an empty `OnceOption` cannot be re-set to contain a
value.

Additionally, `OnceOption` implements `Deref`
and `DerefMut` so that its contents can be
accessed without pattern-matching (but implicitly unwrapping).

It supports comparisons (`PartialEq`, `Eq`, `Ord` or
`PartialOrd`) with other `OnceOption` containing the same type,
as long as the contained type also implements those traits.
Furthermore, it can be used as a hash key if the cotnained type
is `Hash`.

It supports being displayed if the contained type is `Display`,
and forwards all the formatting traits (except `Debug` and
`Pointer`) to its contained-type.

# Rationale

The main, but not only, purpose of `OnceOption` is to simplify
and regulate the dropping of members that have methods consuming
the values.

As an example, this code will fail to compile:

```compile_fail
// Warning: this code does *NOT* compile

use std::{thread, time::Duration};

struct SomeType {
    handle: thread::JoinHandle<u32>,
}

impl SomeType {
    fn new() -> Self {
        Self {
            handle: thread::spawn(|| {
                thread::sleep(Duration::from_secs(5));
                42
            }),
        }
    }
}

impl Drop for SomeType {
    fn drop(&mut self) {
        println!("The answer is {}", self.handle.join());
    }
}
```
The compiler will fail with an error like ([try it!](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=39a859f7169e43de84a01d67ec0ca685)):

```text
Compiling playground v0.0.1 (/playground)
error[E0507]: cannot move out of `self.handle` which is behind a mutable reference
    --> src/lib.rs:22:38
     |
22   |         println!("The answer is {}", self.handle.join().unwrap());
     |                                      ^^^^^^^^^^^ ------ `self.handle` moved due to this method call
     |                                      |
     |                                      move occurs because `self.handle` has type `JoinHandle<u32>`, which does not implement the `Copy` trait
     |
note: `JoinHandle::<T>::join` takes ownership of the receiver `self`, which moves `self.handle`
    --> /playground/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/lib/rustlib/src/rust/library/core/src/thread/mod.rs:1649:17
     |
1649 |     pub fn join(self) -> Result<T> {
     |                 ^^^^

For more information about this error, try `rustc --explain E0507`.
error: could not compile `playground` (lib) due to 1 previous error

```

`OnceOption`  can be used to fix the issue with minimal changes to the
code:

```rust
use once_option::OnceOption;
use std::{thread, time::Duration};

struct SomeType {
    handle: OnceOption<thread::JoinHandle<u32>>,
}

impl SomeType {
    fn new() -> Self {
        Self {
            handle: thread::spawn(|| {
                thread::sleep(Duration::from_secs(5));
                42
            }).into(),
        }
    }

    fn thread_id(&self) -> thread::ThreadId {
        self.handle.thread().id()
    }
}

impl Drop for SomeType {
    fn drop(&mut self) {
        println!("The answer is {}", self.handle.take().join().unwrap());
    }
}
```

# Representation
`OnceOption<T>` has the same ABI of `Option<T>`; this means that
`OnceOption<T>` has the same size, alignment, and function call ABI as `Option<T>`.

An implication of this, is that all the ABI guarantees that `Option<T>` makes
(i.e. being transmutable from a T under some conditions), also apply to
`OnceOption<T>`. For further details, see [the documentation for `Option`](https://doc.rust-lang.org/std/option/index.html#representation).

