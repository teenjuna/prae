#![cfg_attr(docsrs, feature(doc_cfg))]

//! `prae` is a crate that aims to provide a better way to define types that
//! require validation.
//!
//! The main concept of the library is the [`Wrapper`](crate::Wrapper) trait.
//! This trait describes a
//! [`Newtype`](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)
//! wrapper struct that contains some inner value and provides methods to
//! construct, read and mutate it.
//!
//! The easiest way to create a type that implements [`Wrapper`](crate::Wrapper)
//! is to use [`define!`](crate::define) and [`extend!`](crate::extend) macros.
//!
//! # Example
//! Suppose you want to create a type `Username`. You want this type to be a
//! `String`, and you don't want it to be empty. Traditionally, you would create
//! a wrapper struct with getter and setter functions, like this simplified
//! example:
//! ```
//! #[derive(Debug)]
//! pub struct Username(String);
//!
//! impl Username {
//!     pub fn new(username: &str) -> Result<Self, &'static str> {
//!         let username = username.trim().to_owned();
//!         if username.is_empty() {
//!             Err("value is invalid")
//!         } else {
//!             Ok(Self(username))
//!         }
//!     }
//!
//!     pub fn get(&self) -> &str {
//!         &self.0
//!     }
//!
//!     pub fn set(&mut self, username: &str) -> Result<(), &'static str> {
//!         let username = username.trim().to_owned();
//!         if username.is_empty() {
//!             Err("value is invalid")
//!         } else {
//!             self.0 = username;
//!             Ok(())
//!         }
//!    }
//! }
//!
//! let username = Username::new(" my username ").unwrap();
//! assert_eq!(username.get(), "my username");
//!
//! let err = Username::new("  ").unwrap_err();
//! assert_eq!(err, "value is invalid");
//! ```
//!
//! Using `prae`, you will do it like this:
//! ```
//! use prae::Wrapper;
//!
//! prae::define! {
//!     #[derive(Debug)]
//!     pub Username: String;
//!     adjust |username| *username = username.trim().to_owned();
//!     ensure |username| !username.is_empty();
//! }
//!
//! let username = Username::new(" my username ").unwrap();
//! assert_eq!(username.get(), "my username");
//!
//! let err = Username::new("  ").unwrap_err();
//! assert_eq!(err.original, "value is invalid");
//! assert_eq!(err.value, "");
//! ```
//!
//! Futhermore, `prae` allows you to use custom errors and extend your types.
//! See docs for [`define!`](crate::define) and [`extend!`](crate::define) for
//! more information and examples.
//!
//! # Compilation speed
//!
//! The macros provided by this crate are declarative, therefore make almost
//! zero impact on the compilation speed.
//!
//! # Performarnce impact
//!
//! If you find yourself in a situation where the internal adjustment and
//! validation of your type becomes a performance bottleneck (for example, you
//! perform a heavy validation and mutate your type in a hot loop) - try
//! `_unprocessed` variants of [`Wrapper`] methods. They won't call
//! [`Wrapper::PROCESS`]. However, I strongly advise you to call
//! [`Wrapper::verify`] after such operations.
//!
//! # Feature flags
//!
//!  `prae` provides additional features:
//!
//!  Name | Description
//!  ---|---
//!  `serde` | Adds the [`impl_serde`] plugin.
//!  `unprocessed` | Adds the `_unprocessed` methods to [`Wrapper`].
//!
//! # Credits
//! This crate was highly inspired by the
//! [tightness](https://github.com/PabloMansanet/tightness) crate. It's basically
//! just a fork of tightness with a slightly different philosophy.
//! See [this](https://github.com/PabloMansanet/tightness/issues/2) issue for details.

mod core;
pub use crate::core::*;

/// Convenience macro that creates a
/// [`Newtype`](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)
/// wrapper struct that implements [`Wrapper`].
///
/// The macro accepts several arguments (see [macro structure](#macro-structure)
/// for more info). By default, it generates a bare minimum of code:
/// - The `Newtype` struct;
/// - The implementation of the [`Wrapper`] for the struct;
/// - The implementation of the [`AsRef`](AsRef);
/// [`Borrow`](std::borrow::Borrow),
/// [`TryFrom`](TryFrom) and [`From`](From) traits for the struct.
///
/// However, the generated code can be extended in using two methods:
/// - Attribute macros attached to the type signature (e.g. `#[derive(Debug)]`);
/// - Type plugins specified in the end of the macro.
///
/// It is worth noting that the inner value of created `Newtype` struct can be
/// accessed from the code in the same module. To fully protect this value from
/// being accessed directly, put your type in a separate module.
///
/// # Macro structure
///
/// Table of contents:
/// - [Type signature](#type-signature)
/// - [`adjust` closure](#adjust-closure)
/// - [`ensure` closure](#ensure-closure)
/// - [`validate` closure](#validate-closure)
/// - [Plugins](#plugins)
///
/// ## Type signature
///
/// This is the only required argument of the macro. It specifies the visibiliy
/// and the name of the created struct, as well as it's inner type. For
/// example, this
/// ```
/// prae::define! {
///     /// An ID of a user.
///     pub UserID: i64;
/// }
///
/// prae::define! {
///     /// A name of a user.
///     Username: String;
/// }
/// ```
/// will expand into this:
/// ```
/// /// An ID of a user.
/// pub struct UserID(i64);
/// // other impls...
///
/// /// A name of a user.
/// struct Username(String);
/// // other impls...
/// ```
/// You could also use attribute macros on top of your signature if you like.
/// For example, this
/// ```
/// prae::define! {
///     #[derive(Debug, Clone)]
///     pub Username: String;
/// }
/// ```
/// will expand into this:
/// ```
/// #[derive(Debug, Clone)]
/// pub struct Username(String);
/// // other impls...
/// ```
/// Meaning that your type now implements `Debug` and `Clone`.
///
/// **Note**: check out
/// [`derive_more`](https://docs.rs/derive_more/latest/derive_more/)
/// for more derive macros.
///
/// # `adjust` closure
///
/// This argument specifies a closure that will be executed on every
/// construction and mutation of the wrapper to make sure that it's value is
/// adjusted properly. For example:
/// ```
/// use prae::Wrapper;
///
/// prae::define! {
///     #[derive(Debug)]
///     pub Text: String;
///     adjust |text: &mut String| *text = text.trim().to_owned();
/// }
///
/// let mut text = Text::new("   hello world!   ").unwrap();
/// assert_eq!(text.get(), "hello world!");
///
/// text.set("   new value\n\n\n").unwrap();
/// assert_eq!(text.get(), "new value");
/// ```
///
/// # `ensure` closure
///
/// This argument specifies a closure that will be executed on every
/// construction and mutation of the wrapper to make sure that it's value is
/// valid. For example:
/// ```
/// use prae::Wrapper;
///
/// prae::define! {
///     #[derive(Debug)]
///     pub Text: String;
///     ensure |text: &String| !text.is_empty();
/// }
///
/// assert!(Text::new("hello world").is_ok());
/// assert!(Text::new("").is_err());
/// ```
/// As you can see, the closure receives a shared reference to the inner value
/// and returns `true` if the value is valid, and `false` if it's not.
///
/// This closure is easy to use, but it has a downside: you can't customize your
/// error type. The [`Wrapper::Error`] type will always
/// be a `&'static str` with a generic error message:
/// ```
/// # use prae::Wrapper;
/// # prae::define! {
/// #     #[derive(Debug)]
/// #     pub Text: String;
/// #     ensure |text: &String| !text.is_empty();
/// # }
/// let err = Text::new("").unwrap_err();
/// assert_eq!(err.original, "value is invalid");
/// assert_eq!(
///     err.to_string(),
///     "failed to construct type Text from value \"\": value is invalid",
/// )
/// ```
/// If you want more control, use [`validate` closure](#validate-closure)
/// closure described below.
///
/// **Note**:
/// - this closure can be used together with the [`adjust`
///   closure](#adjust-closure) and will be executed after it;
/// - this closure can't be used together with the [`validate`
///   closure](#validate-closure).
///
/// # `validate` closure
/// This closure is similar to the [`ensure` closure](#ensure-closure), but uses
/// custom error specified by the user:
/// ```
/// use std::fmt;
/// use prae::Wrapper;
///
/// #[derive(Debug)]
/// pub enum TextError {
///     Empty,
/// }
///
/// // Required in order for `err.to_string()` to work.
/// impl fmt::Display for TextError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "{}", match self {
///             Self::Empty => "text is empty",
///         })
///     }
/// }
///
/// prae::define! {
///     #[derive(Debug)]
///     pub Text: String;
///     validate(TextError) |text: &String| {
///         if text.is_empty() {
///             Err(TextError::Empty)
///         } else {
///             Ok(())
///         }
///     };
/// }
///
/// let err = Text::new("").unwrap_err();
/// assert!(matches!(err.original, TextError::Empty));
/// assert_eq!(
///     err.to_string(),
///     "failed to construct type Text from value \"\": text is empty",
/// )
/// ```
/// As you can see, the closure receives a shared reference to the inner value
/// and returns `Ok(())` if the value is valid, and `Err(...)` if it’s not.
///
/// **Note**:
/// - this closure can be used together with the [`adjust`
///   closure](#adjust-closure) and will be executed after it;
/// - this closure can't be used together with the [`ensure`
///   closure](#ensure-closure).
///
/// # Plugins
///
/// Sometimes attribute macros just dont't cut it. In this case, you have two
/// options:
/// - add manual `impl` to your type;
/// - use plugins.
///
/// In the context of this macro, plugin is just a macro that takes your type as
/// an input and does something with it.
///
/// For example, suppose we want our type to implement
/// [`serde::Serialize`](::serde::Serialize) and
/// [`serde::Deserialize`](::serde::Deserialize). We *could* use attribute
/// macros:
/// ```
/// use serde::{Serialize, Deserialize};
///
/// prae::define! {
///     #[derive(Serialize, Deserialize)]
///     pub Username: String;
///     adjust |un| *un = un.trim().to_owned();
///     ensure |un| !un.is_empty();
/// }
/// ```
/// However, this implementation won't use our `adjust` and `ensure` closures
/// for the type deserialization. This means, that we can create `Username` with
/// invalid data:
/// ```
/// # use prae::Wrapper;
/// # use serde::{Serialize, Deserialize};
/// # prae::define! {
/// #     #[derive(Serialize, Deserialize)]
/// #     pub Username: String;
/// #     adjust |un| *un = un.trim().to_owned();
/// #     ensure |un| !un.is_empty();
/// # }
/// // This won't work
/// assert!(Username::new("   ").is_err());
///
/// // But this will
/// let un: Username = serde_json::from_str("\"   \"").unwrap();
/// assert_eq!(un.get(), "   "); // not good
/// ```
/// To avoid this, we need to add a custom implementation of
/// [`serde::Deserialize`](::serde::Deserialize) for our type. Since the
/// implementation is indentical for any type created with this crate, `prae`
/// ships with a built-in (under `serde` feature) plugin called
/// [`impl_serde`]. This plugin will implement both
/// [`serde::Serialize`](::serde::Serialize) and
/// [`serde::Deserialize`](::serde::Deserialize) the right way:
/// ```
/// use prae::Wrapper;
/// use serde::{Serialize, Deserialize};
///
/// prae::define! {
///     #[derive(Debug)]
///     pub Username: String;
///     adjust |un| *un = un.trim().to_owned();
///     ensure |un| !un.is_empty();
///     plugins: [
///         prae::impl_serde,
///     ];
/// }
///
/// // This will work
/// let un: Username = serde_json::from_str("\"  qwerty \"").unwrap();
/// assert_eq!(un.get(), "qwerty");
///
/// // But this won't
/// let err = serde_json::from_str::<Username>("\"   \"").unwrap_err();
/// assert_eq!(err.to_string(), "value is invalid");
/// ```
/// You can implement your own plugins and use them for your types - it's easy.
#[macro_export]
macro_rules! define {
    // Required part:
    // - Optional attribute macro;
    // - Required type signature;
    // - Optional closures.
    // - Optional plugins.
    {
        $(#[$meta:meta])*
        $vis:vis $wrapper:ident: $inner:ty;
        $(adjust $adjust:expr;)?
        $(ensure $ensure:expr;)?
        $(validate($err:ty) $validate:expr;)?
        $(plugins: [$($plugin:path),+ $(,)?];)?
    } => {
        $(#[$meta])*
        $vis struct $wrapper($inner);
        impl $crate::Wrapper for $wrapper {
            const NAME: &'static str = stringify!($wrapper);
            type Inner = $inner;
            $crate::define!(
                $(adjust $adjust;)?
                $(ensure $ensure;)?
                $(validate($err) $validate;)?
            );
            $crate::__impl_wrapper_methods!();
        }
        $crate::__impl_external_traits!($wrapper, $inner);
        $($($plugin!($wrapper);)*)?
    };
    // Optional closures 1:
    // - Optional `adjust` closure.
    {
        $(adjust $adjust:expr;)?
    } => {
        type Error = std::convert::Infallible;
        const PROCESS: fn(&mut Self::Inner) -> Result<(), Self::Error> = |mut _v| {
            $({
                let adjust: fn(&mut Self::Inner) = $adjust;
                adjust(&mut _v);
            })?
            Ok(())
        };
    };
    // Optional closures 2:
    // - Optional `adjust` closure.
    // - Required `ensure` closure.
    {
        $(adjust $adjust:expr;)?
        ensure $ensure:expr;
    } => {
        type Error = &'static str;
        const PROCESS: fn(&mut Self::Inner) -> Result<(), Self::Error> = |mut _v| {
            $({
                let adjust: fn(&mut Self::Inner) = $adjust;
                adjust(&mut _v);
            })?
            {
                let ensure: fn(&Self::Inner) -> bool = $ensure;
                if !ensure(&_v) {
                    return Err("value is invalid")
                }
            }
            Ok(())
        };
    };
    // Optional closures 3:
    // - Optional `adjust` closure.
    // - Required `validate` closure.
    {
        $(adjust $adjust:expr;)?
        validate($err:ty) $validate:expr;
    } => {
        type Error = $err;
        const PROCESS: fn(&mut Self::Inner) -> Result<(), Self::Error> = |mut _v| {
            $({
                let adjust: fn(&mut Self::Inner) = $adjust;
                adjust(&mut _v);
            })?
            {
                let validate: fn(&Self::Inner) -> Result<(), Self::Error> = $validate;
                if let Err(err) = validate(&_v) {
                    return Err(err)
                }
            }
            Ok(())
        };
    }
}

/// Convenience macro that creates a
/// [`Newtype`](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)
/// wrapper struct that implements [`Wrapper`] and extends another [`Wrapper`].
///
/// The usage of the macro is identical to the [`define!`](crate::define), so
/// check out it's documentation to learn more. The only difference is the fact
/// that the inner type specified in the type signature must implement
/// [`Wrapper`].
///
/// The created struct will inherit the inner type of that another wrapper, and
/// also will run that another wrapper's adjustment and validation closures
/// before it's own adjustment and validation closures. For example:
/// ```
/// use prae::Wrapper;
///
/// prae::define! {
///     pub Text: String;
///     adjust |text| *text = text.trim().to_owned();
///     ensure |text| !text.is_empty();
/// }
///
/// prae::extend! {
///     #[derive(Debug)]
///     pub Sentence: Text;
///     ensure |sentence: &String| {
///         // Note that `sentence` is a `&String`, not `&Text`!
///         // It's value is already trimmed and checked for emptiness.
///         // Now we only need to check conditions that are important
///         // for the new type
///         sentence.ends_with(&['.', '!', '?'][..])
///     };
/// }
///
/// // It works
/// let sentence = Sentence::new("   My sentence! ").unwrap();
/// assert_eq!(sentence.get(), "My sentence!");
///
/// // Doesn't pass the validation of `Text`
/// assert!(Sentence::new("   ").is_err());
///
/// // Doesn't pass the validation of `Sentence`
/// assert!(Sentence::new("Without punctuation").is_err());
/// ```
#[macro_export]
macro_rules! extend {
    // Required part:
    // - Optional attribute macro;
    // - Required type signature;
    // - Optional closures.
    // - Optional plugins.
    {
        $(#[$meta:meta])*
        $vis:vis $wrapper:ident: $inner:ty;
        $(adjust $adjust:expr;)?
        $(ensure $ensure:expr;)?
        $(validate($err:ty) $validate:expr;)?
        $(plugins: [$($plugin:path),+ $(,)?];)?

    } => {
        $(#[$meta])*
        $vis struct $wrapper(<$inner as $crate::Wrapper>::Inner);
        impl $crate::Wrapper for $wrapper {
            const NAME: &'static str = stringify!($wrapper);
            type Inner = <$inner as $crate::Wrapper>::Inner;
            $crate::extend!(
                $inner;
                $(adjust $adjust;)?
                $(ensure $ensure;)?
                $(validate($err) $validate;)?
            );
            $crate::__impl_wrapper_methods!();
        }
        $crate::__impl_external_traits!($wrapper, <$inner as $crate::Wrapper>::Inner);
        $($($plugin!($wrapper);)*)?
    };
    // Optional closures 1:
    // - Optional `adjust` closure.
    {
        $inner:ty;
        $(adjust $adjust:expr;)?
    } => {
        type Error = &'static str;
        const PROCESS: fn(&mut Self::Inner) -> Result<(), Self::Error> = |mut _v| {
            <$inner as $crate::Wrapper>::PROCESS(&mut _v)?;
            $({
                let adjust: fn(&mut Self::Inner) = $adjust;
                adjust(&mut _v);
            })?
            Ok(())
        };
    };
    // Optional closures 2:
    // - Optional `adjust` closure.
    // - Required `ensure` closure.
    {
        $inner:ty;
        $(adjust $adjust:expr;)?
        ensure $ensure:expr;
    } => {
        type Error = &'static str;
        const PROCESS: fn(&mut Self::Inner) -> Result<(), Self::Error> = |mut _v| {
            <$inner as $crate::Wrapper>::PROCESS(&mut _v)?;
            $({
                let adjust: fn(&mut Self::Inner) = $adjust;
                adjust(&mut _v);
            })?
            {
                let ensure: fn(&Self::Inner) -> bool = $ensure;
                if !ensure(&_v) {
                    return Err("value is invalid")
                }
            }
            Ok(())
        };
    };
    // Optional closures 3:
    // - Optional `adjust` closure.
    // - Required `validate` closure.
    {
        $inner:ty;
        $(adjust $adjust:expr;)?
        validate($err:ty) $validate:expr;
    } => {
        type Error = $err;
        const PROCESS: fn(&mut Self::Inner) -> Result<(), Self::Error> = |mut _v| {
            <$inner as $crate::Wrapper>::PROCESS(&mut _v)?;
            $({
                let adjust: fn(&mut Self::Inner) = $adjust;
                adjust(&mut _v);
            })?
            {
                let validate: fn(&Self::Inner) -> Result<(), Self::Error> = $validate;
                if let Err(err) = validate(&_v) {
                    return Err(err)
                }
            }
            Ok(())
        };
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_wrapper_methods {
    () => {
        fn new(value: impl Into<Self::Inner>) -> Result<Self, $crate::ConstructionError<Self>> {
            let mut value = value.into();
            match Self::PROCESS(&mut value) {
                Ok(()) => Ok(Self(value)),
                Err(original) => Err($crate::ConstructionError { original, value }),
            }
        }
        fn get(&self) -> &Self::Inner {
            &self.0
        }
        fn set(
            &mut self,
            value: impl Into<Self::Inner>,
        ) -> Result<(), $crate::ConstructionError<Self>> {
            let mut value = value.into();
            match Self::PROCESS(&mut value) {
                Ok(()) => {
                    self.0 = value;
                    Ok(())
                }
                Err(original) => Err($crate::ConstructionError { original, value }),
            }
        }
        fn __mutate_with(
            &mut self,
            clone: impl Fn(&Self::Inner) -> Self::Inner,
            f: impl FnOnce(&mut Self::Inner),
        ) -> Result<(), $crate::MutationError<Self>> {
            let mut value = clone(&self.0);
            f(&mut value);
            match Self::PROCESS(&mut value) {
                Ok(()) => {
                    self.0 = value;
                    Ok(())
                }
                Err(original) => Err($crate::MutationError {
                    original,
                    old_value: clone(&self.0),
                    new_value: value,
                }),
            }
        }
        #[cfg(feature = "unprocessed")]
        fn new_unprocessed(value: impl Into<Self::Inner>) -> Self {
            let mut value = value.into();
            debug_assert!(Self::PROCESS(&mut value).is_ok());
            Self(value)
        }
        #[cfg(feature = "unprocessed")]
        fn set_unprocessed(&mut self, value: impl Into<Self::Inner>) {
            let mut value = value.into();
            debug_assert!(Self::PROCESS(&mut value).is_ok());
            self.0 = value;
        }
        #[cfg(feature = "unprocessed")]
        fn mutate_unprocessed(&mut self, f: impl FnOnce(&mut Self::Inner)) {
            f(&mut self.0);
            debug_assert!(Self::PROCESS(&mut self.0).is_ok());
        }
        #[cfg(feature = "unprocessed")]
        fn verify(mut self) -> Result<Self, $crate::VerificationError<Self>> {
            match Self::PROCESS(&mut self.0) {
                Ok(()) => Ok(self),
                Err(original) => Err($crate::VerificationError {
                    original,
                    value: self.0,
                }),
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_external_traits {
    ($wrapper:ident, $inner:ty) => {
        impl std::convert::AsRef<$inner> for $wrapper {
            fn as_ref(&self) -> &$inner {
                &self.0
            }
        }
        impl std::borrow::Borrow<$inner> for $wrapper {
            fn borrow(&self) -> &$inner {
                &self.0
            }
        }
        impl std::convert::TryFrom<$inner> for $wrapper {
            type Error = $crate::ConstructionError<$wrapper>;
            fn try_from(value: $inner) -> Result<Self, Self::Error> {
                <$wrapper as $crate::Wrapper>::new(value)
            }
        }
        impl std::convert::From<$wrapper> for $inner {
            fn from(wrapper: $wrapper) -> Self {
                wrapper.0
            }
        }
    };
}

/// Implement [`serde::Serialize`](::serde::Serialize) and
/// [`serde::Deserialize`](::serde::Deserialize) for the wrapper. Deserilization
/// will fail if the value doesn't pass wrapper's
/// [`PROCESS`](crate::Wrapper::PROCESS) function.
///
/// For this to work, the inner type of the wrapper must also implement these
/// traits.
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
#[macro_export]
macro_rules! impl_serde {
    ($wrapper:ident) => {
        impl<'de> ::serde::Deserialize<'de> for $wrapper
        where
            Self: $crate::Wrapper + std::fmt::Debug,
            <Self as $crate::Wrapper>::Inner: ::serde::Deserialize<'de> + std::fmt::Debug,
            <Self as $crate::Wrapper>::Error: std::fmt::Display + std::fmt::Debug,
        {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                <Self as $crate::Wrapper>::new(<Self as $crate::Wrapper>::Inner::deserialize(
                    deserializer,
                )?)
                .map_err(|err| ::serde::de::Error::custom(err.original))
            }
        }
        impl ::serde::Serialize for $wrapper
        where
            Self: $crate::Wrapper + std::fmt::Debug,
            <Self as $crate::Wrapper>::Inner: ::serde::Serialize,
            <Self as $crate::Wrapper>::Error: std::fmt::Display + std::fmt::Debug,
        {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                <Self as $crate::Wrapper>::Inner::serialize(&self.0, serializer)
            }
        }
    };
}
