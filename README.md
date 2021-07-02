# prae

[![crates.io version](https://shields.io/crates/v/prae)](https://crates.io/crates/prae)
[![docs.rs](https://docs.rs/prae/badge.svg)](https://docs.rs/prae)
[![crates.io license](https://shields.io/crates/l/prae)](https://crates.io/crates/prae)

This crate provides a convenient macro that allows you to generate type wrappers that promise to always hold arbitrary invariants specified by you.

# Usage

Let's create a `Username` type. It will be a wrapper around non-empty `String`:

```rust
prae::define!(pub Username: String ensure |u| !u.is_empty());

// We can't create an invalid username.
assert!(Username::new("").is_err());

// But we can create a valid one!
let mut u = Username::new("valid name").unwrap();
assert_eq!(u.get(), "valid name");

// We can mutate it:
assert!(u.try_mutate(|u| *u = "new name".to_owned()).is_ok());
assert_eq!(u.get(), "new name"); // our name has changed!

// But we can't make it invalid:
assert!(u.try_mutate(|u| *u = "".to_owned()).is_err());
assert_eq!(u.get(), "new name"); // our name hasn't changed!

// Let's try this...
assert!(Username::new("  ").is_ok()); // looks kind of invalid though :(
```

As you can see, the last example treats `"  "` as a valid username, but it's not. We
can of course do something like `Username::new(s.trim())` every time, but why should
we do it ourselves? Let's automate it!

```rust
prae::define! {
    pub Username: String
    adjust |u| *u = u.trim().to_string()
    ensure |u| !u.is_empty()
}

let mut u = Username::new(" valid name \n\n").unwrap();
assert_eq!(u.get(), "valid name"); // now we're talking!

// This also works for mutations:
assert!(matches!(u.try_mutate(|u| *u = "   ".to_owned()), Err(prae::ValidationError)));
```

Now our `Username` trims provided value automatically.

You might noticed that `prae::ValidationError` is returned by default when our
construction/mutation fails. Altough it's convenient, there are situations when you might
want to return a custom error. And `prae` can help with this:

```rust
#[derive(Debug)]
struct UsernameError;

prae::define! {
    pub Username: String
    adjust   |u| *u = u.trim().to_string()
    validate |u| -> Option<UsernameError> {
        if u.is_empty() {
            Some(UsernameError)
        } else {
            None
        }
    }
}

assert!(matches!(Username::new("  "), Err(UsernameError)));
```

Perfect! Now you can integrate it in your code and don't write `.map_err(...)` everywhere.

# Drawbacks
Although proc macros are very powerful, they aren't free. In this case, you have to pull up additional dependencies such as `syn` and `quote`, and expect a *slightly* slower compile times.

# Credits
This crate was highly inspired by the [tightness](https://github.com/PabloMansanet/tightness) crate. It's basically just a fork of tightness with a slightly different philosophy. See [this](https://github.com/PabloMansanet/tightness/issues/2) issue for details.
