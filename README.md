# prae

[![crates.io version](https://shields.io/crates/v/prae)](https://crates.io/crates/prae)
[![docs.rs](https://docs.rs/prae/badge.svg)](https://docs.rs/prae)
[![crates.io license](https://shields.io/crates/l/prae)](https://crates.io/crates/prae)

`prae` is a crate that aims to provide a better way to define types that
require validation.

The main concept of the library is the [`Wrapper`](crate::Wrapper) trait.
This trait describes a
[`Newtype`](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)
wrapper struct that contains some inner value and provides methods to
construct, read and mutate it.

The easiest way to create a type that implements [`Wrapper`](crate::Wrapper)
is to use [`define!`](crate::define) and [`extend!`](crate::extend) macros.

## Example

Suppose you want to create a type `Username`. You want this type to be a
`String`, and you don't want it to be empty. Traditionally, you would create
a wrapper struct with getter and setter functions, like this simplified
example:

```rust
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

```rust
use prae::Wrapper;

prae::define! {
    #[derive(Debug)]
    pub Username: String;
    adjust |username| *username = username.trim().to_owned();
    ensure |username| !username.is_empty();
}

let username = Username::new(" my username ").unwrap();
assert_eq!(username.get(), "my username");

let err = Username::new("  ").unwrap_err();
assert_eq!(err.original, "value is invalid");
assert_eq!(err.value, "");
```

Futhermore, `prae` allows you to use custom errors and extend your types.
See docs for [`define!`](crate::define) and [`extend!`](crate::define) for
more information and examples.

## Compilation speed

The macros provided by this crate are declarative, therefore make almost
zero impact on the compilation speed.

## Performarnce impact

If you find yourself in a situation where the internal adjustment and
validation of your type becomes a performance bottleneck (for example, you
perform a heavy validation and mutate your type in a hot loop) - try
`_unprocessed` variants of `Wrapper` methods. They won't call
`Wrapper::PROCESS`. However, I strongly advise you to call
`Wrapper::verify` after such operations.

## Feature flags

`prae` provides additional features:

| Name          | Description                                   |
| ------------- | --------------------------------------------- |
| `serde`       | Adds the `impl_serde` plugin.                 |
| `unprocessed` | Adds the `_unprocessed` methods to `Wrapper`. |

## Credits

This crate was highly inspired by the
[tightness](https://github.com/PabloMansanet/tightness) crate. It's basically
just a fork of tightness with a slightly different philosophy.
See [this](https://github.com/PabloMansanet/tightness/issues/2) issue for details.

License: Unlicense
