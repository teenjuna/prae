#![cfg_attr(docsrs, feature(doc_cfg))]

mod core;
pub use crate::core::*;

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
        $(plugins: [$($feature:path),+ $(,)?])?$(;)?
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
        $($($feature!($wrapper);)*)?
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
        $(plugins: [$($feature:path),+ $(,)?])?$(;)?

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
        $($($feature!($wrapper);)*)?
    };
    // Optional closures 1:
    // - Optional `adjust` closure.
    {
        $inner:ty;
        $(adjust $adjust:expr;)?
    } => {
        // type Error = <$inner as $crate::Wrapper>::Error;
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
            $({
                let ensure: fn(&Self::Inner) -> bool = $ensure;
                if !ensure(&_v) {
                    return Err("value is invalid")
                }
            })?
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
                Err(inner) => Err($crate::ConstructionError { inner, value }),
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
                Err(inner) => Err($crate::ConstructionError { inner, value }),
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
                Err(inner) => Err($crate::MutationError {
                    inner,
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
                Err(inner) => Err($crate::VerificationError {
                    inner,
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
///
/// **NOTE:** this macro is only available with the `serde` feature enabled.
#[cfg(feature = "serde")]
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
                .map_err(|err| ::serde::de::Error::custom(err.inner))
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
