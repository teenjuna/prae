# prae

[![crates.io version](https://shields.io/crates/v/prae)](https://crates.io/crates/prae)
[![docs.rs](https://docs.rs/prae/badge.svg)](https://docs.rs/prae)
[![crates.io license](https://shields.io/crates/l/prae)](https://crates.io/crates/prae)

# What is prae?

This crate aims to provide a better way to define types that require
validation. `prae` **is not** a validation library, but a library that
**helps developers** to define validation-requiring types with **very little
effort**.

# How it works?

The main way to use `prae` is through [`define!`](https://docs.rs/prae/latest/prae/macro.define.html) macro.

For example, suppose you want to create a `Username` type. You want this
type to be a string, and you don't want it to be empty. Traditionally, would
create a wrapper struct with getter and setter functions, like this
simplified example:
```
#[derive(Debug)]
pub struct Username(String);

impl Username {
    pub fn new(username: &str) -> Result<Self, &'static str> {
        let username = username.trim().to_owned();
        if username.is_empty() {
            Err("value is invalid")
        } else {
            Ok(Self(username))
        }
    }

    pub fn get(&self) -> &str {
        &self.0
    }

    pub fn set(&mut self, username: &str) -> Result<(), &'static str> {
        let username = username.trim().to_owned();
        if username.is_empty() {
            Err("value is invalid")
        } else {
            self.0 = username;
            Ok(())
        }
   }
}

let username = Username::new(" my username ").unwrap();
assert_eq!(username.get(), "my username");

let err = Username::new("  ").unwrap_err();
assert_eq!(err, "value is invalid");
```

Using `prae`, you will do it like this:
```
use prae::define;

define! {
    pub Username: String
    adjust |username| *username = username.trim().to_owned()
    ensure |username| !username.is_empty()
}

let username = Username::new(" my username ").unwrap();
assert_eq!(username.get(), "my username");

let err = Username::new("  ").unwrap_err();
assert_eq!(err.inner, "value is invalid");
assert_eq!(err.value, "");
```

Futhermore, `prae` allows you to use custom errors and extend your types.
See [docs](https://docs.rs/prae/latest/prae/index.html) for more information and examples.

# Credits
This crate was highly inspired by the [tightness](https://github.com/PabloMansanet/tightness) crate. It's basically just a fork of tightness with a slightly different philosophy. See [this](https://github.com/PabloMansanet/tightness/issues/2) issue for details.
