//! This crate provides a way to define type wrappers (guards) that behave as close as
//! possible to the underlying type, but guarantee to uphold arbitrary invariants at all times.
//!
//! The name comes from latin _praesidio_, which means _guard_.
//!
//! # Basic usage
//!
//! The simplest way to create a [guard](Guard) is to use the [`define!`](prae_macro::define) macro.
//!
//! Let's create a `Text` type. It will be a wrapper around a `String` with an invariant that the
//! value is not empty:
//!
//! ```
//! use prae::{define, Guard};
//!
//! // Here we define our new type.
//! // `pub`             - the visibility of our type;
//! // `Text`            - the name of our type;
//! // `String`          - the underlying type;
//! // `ensure $closure` - the closure that returns `true` if the value is valid.
//! define!(pub Text: String ensure |t| !t.is_empty());
//!
//! // We can easily create a value of our type. Note that the type of the agrument
//! // is not `String`. That is because `Text::new(...)` accepts anything
//! // that is `Into<String>`.
//! let mut t = Text::new("not empty!").unwrap();
//! assert_eq!(t.get(), "not empty!");
//!
//! // One way to mutate the value is to call the `mutate` method.
//! // See docs for `prae::Guard` to learn about other methods.
//! t.mutate(|t| *t = format!("{} updated", t));
//! assert_eq!(t.get(), "not empty! updated");
//!
//! // Creating an invalid value is not possible.
//! assert!(Text::new("").is_err());
//! ```
//!
//! Okay, we're getting there. Right now our type will accept every non-empty string and reject any
//! zero-length string. But does it really protect us from a possible abuse? The following example
//! shows us, that the answer is no:
//!
//! ```
//! # use prae::{define, Guard};
//! # define!(pub Text: String ensure |t| !t.is_empty());
//! // Technically not empty, but makes no sense as a "text" for a human.
//! assert!(Text::new(" \n\n ").is_ok());
//!
//! // Trailing whitespace should not be allowed either.
//! assert!(Text::new(" text ").is_ok());
//! ```
//!
//! One way to solve this is to tell the user of our type to always trim the string before any
//! construction/mutation of the `Text`. But this can go wrong very easily. It would be better if
//! we could somehow embed this behaviour into our type. And we can!
//!
//! ```
//! use prae::{define, Guard};
//!
//! define! {
//!     /// Btw, this comment will document our type!
//!     pub Text: String
//!     // We will mutate given value before every construction/mutation.
//!     adjust |t| {
//!         let trimmed = t.trim();
//!         if trimmed.len() != t.len() {
//!             *t = trimmed.to_string()
//!         }
//!     }
//!     ensure |t| !t.is_empty()
//! }
//!
//! // This won't work anymore.
//! assert!(Text::new("   ").is_err());
//!
//! // And this will be automatically adjusted.
//! let t = Text::new(" no trailing whitespace anymore! ").unwrap();
//! assert_eq!(t.get(), "no trailing whitespace anymore!");
//! ```
//!
//! That's a lot better!
//!
//! # Custom errors
//!
//! Both [`ConstructionError`](ConstructionError) and [`MutationError`](MutationError) are
//! useful wrappers around some inner error. They provide access to the values that were in play when
//! the error occured. By default (when you use `ensure` closure inside the
//! [`define!`]), the inner error is just `&'static str` with the default error message.
//!
//! Sometimes, however, we might want to use our own type. In this case, we should use `validate`
//! instead of `ensure`:
//!
//! ```rust
//! use prae::{define, Guard};
//!
//! #[derive(Debug)]
//! pub enum Error {
//!     Empty,
//!     NotEnoughWords,
//!     TooManyWords,
//! }
//!
//! define! {
//!     /// A text that contains two words.
//!     pub TwoWordText: String
//!     adjust   |t| *t = t.trim().to_owned()
//!     validate |t| -> Result<(), Error> {
//!         let wc = t.split_whitespace().count();
//!         if t.is_empty() {
//!             Err(Error::Empty)
//!         } else if wc < 2 {
//!             Err(Error::NotEnoughWords)
//!         } else if wc > 2 {
//!             Err(Error::TooManyWords)
//!         } else {
//!             Ok(())
//!         }
//!     }
//! }
//!
//! assert!(matches!(TwoWordText::new("  ").unwrap_err().inner, Error::Empty));
//! assert!(matches!(TwoWordText::new("word").unwrap_err().inner, Error::NotEnoughWords));
//! assert!(matches!(TwoWordText::new("word word word").unwrap_err().inner, Error::TooManyWords));
//! assert!(TwoWordText::new("word word").is_ok());
//! ```
//!
//! # Extending our types
//!
//! If you want to reuse adjustment/validation behaviour of some type in a new type, you should use
//! [`extend!`]. It's just like [`define!`], but it's inner type should be [`Guard`].
//!
//! ```
//! use prae::{define, extend, Guard};
//!
//! #[derive(Debug)]
//! pub enum TextError {
//!     Empty,
//! }
//!
//! define! {
//!     /// A non-empty string without trailing whitespace.
//!     pub Text: String
//!     adjust   |t| *t = t.trim().to_owned()
//!     validate |t| -> Result<(), TextError> {
//!         if t.is_empty() {
//!             Err(TextError::Empty)
//!         } else {
//!             Ok(())
//!         }
//!     }
//! }
//!
//! #[derive(Debug)]
//! pub enum TwoWordTextError {
//!     Empty,
//!     NotEnoughWords,
//!     TooManyWords,
//! }
//!
//! impl From<TextError> for TwoWordTextError {
//!     fn from(te: TextError) -> Self {
//!         match te {
//!             TextError::Empty => Self::Empty,
//!         }
//!     }
//! }
//!
//! extend! {
//!     /// A text that contains two words.
//!     pub TwoWordText: Text
//!     validate |t| -> Result<(), TwoWordTextError> {
//!         // We don't need to check if `t` is empty, since
//!         // it already passed the validation of `Text` at
//!         // this point.
//!         let wc = t.split_whitespace().count();
//!         if wc < 2 {
//!             Err(TwoWordTextError::NotEnoughWords)
//!         } else if wc > 2 {
//!             Err(TwoWordTextError::TooManyWords)
//!         } else {
//!             Ok(())
//!         }
//!     }
//! }
//!
//! assert!(matches!(TwoWordText::new("  ").unwrap_err().inner, TwoWordTextError::Empty));
//! assert!(matches!(TwoWordText::new("word").unwrap_err().inner, TwoWordTextError::NotEnoughWords));
//! assert!(matches!(TwoWordText::new("word word word").unwrap_err().inner, TwoWordTextError::TooManyWords));
//! assert!(TwoWordText::new("word word").is_ok());
//! ```
//!
//! # Integration with serde
//!
//! If you enable `serde` feature, every [`Guard`] will automatically implement
//! [`Serialize`](https://docs.serde.rs/serde/trait.Serialize.html) and
//! [`Deserialize`](https://docs.serde.rs/serde/trait.Deserialize.html) if its inner type implements them.
//! Deserialization will automatically return an error if the data is not invalid.
//!
//! Here is an example of some API implemented using [`axum`](https://crates.io/crates/axum):
//!
//! ```ignore
//! use prae::{define, Guard};
//! use axum::{extract, handler::post, Router};
//!
//! define! {
//!     Text: String
//!     adjust   |t| *t = t.trim().to_owned()
//!     validate |t| !t.is_empty()
//! }
//!
//! async fn save_text(extract::Json(text): extract::Json<Text>) {
//!     // Our `text` was automatically validated. We don't need to
//!     // do anything manually at all!
//!     // ...
//! }
//!
//! let app = Router::new().route("/texts", post(save_text));
//! ```
//!
//! # Optimising for performance
//!
//! If you find yourself in a situation where always adjusting/validating values of your type
//! is a performance issue, you can opt in to avoid those extra calls using methods under the
//! `unchecked` feature gate. Those methods (`new_unchecked`, `mutate_unchecked`, etc.) don't
//! adjust/validate your values at all, making __you__ responsible for the validity of your data.
//!
//! [`Guard`]: Guard
//! [`define!`]: prae_macro::define
//! [`extend!`]: prae_macro::extend

#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_crate_dependencies)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

mod core;

pub use crate::core::*;
pub use prae_macro::{define, extend};

// We need this to silince the unused_crate_dependencies warning.
// See: https://github.com/rust-lang/rust/issues/57274
#[cfg(test)]
mod test_deps {
    use assert_matches as _;
    use serde as _;
    use serde_json as _;
}
