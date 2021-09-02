# prae

[![crates.io version](https://shields.io/crates/v/prae)](https://crates.io/crates/prae)
[![docs.rs](https://docs.rs/prae/badge.svg)](https://docs.rs/prae)
[![crates.io license](https://shields.io/crates/l/prae)](https://crates.io/crates/prae)

This crate provides a way to define type wrappers (guards) that behave as close as possible to the underlying type, but guarantee to uphold arbitrary invariants at all times. See the [documentation](https://docs.rs/prae) for more examples.

```rust
use prae::{define, Guard};

define! {
    pub Text: String
    adjust |t| {
        let trimmed = t.trim();
        if trimmed.len() != t.len() {
            *t = trimmed.to_string()
        }
    }
    ensure |t| !t.is_empty()
}

let mut t = Text::new("   not empty!   \n\n").unwrap();
assert_eq!(t.get(), "not empty!");

t.mutate(|t| *t = format!("{} updated", t));
assert_eq!(t.get(), "not empty! updated");

assert!(Text::new("").is_err());
```

# Credits
This crate was highly inspired by the [tightness](https://github.com/PabloMansanet/tightness) crate. It's basically just a fork of tightness with a slightly different philosophy. See [this](https://github.com/PabloMansanet/tightness/issues/2) issue for details.
