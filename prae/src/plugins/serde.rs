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
            Self: $crate::Wrapper + ::std::fmt::Debug,
            <Self as $crate::Wrapper>::Inner: ::serde::Deserialize<'de> + ::std::fmt::Debug,
            <Self as $crate::Wrapper>::Error: ::std::fmt::Display + std::fmt::Debug,
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
            Self: $crate::Wrapper + ::std::fmt::Debug,
            <Self as $crate::Wrapper>::Inner: ::serde::Serialize,
            <Self as $crate::Wrapper>::Error: ::std::fmt::Display + std::fmt::Debug,
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
