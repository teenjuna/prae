/// Implement [`Deref`](::core::ops::Deref) for the wrapper.
#[macro_export]
macro_rules! impl_deref {
    ($wrapper:ident) => {
        impl ::core::ops::Deref for $wrapper {
            type Target = <$wrapper as $crate::Wrapper>::Inner;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
    };
}

/// Implement [`Index`](::core::ops::Index) for the wrapper.
#[macro_export]
macro_rules! impl_index {
    ($wrapper:ident) => {
        impl<Idx> ::core::ops::Index<Idx> for $wrapper
        where
            <$wrapper as $crate::Wrapper>::Inner: ::core::ops::Index<Idx>,
        {
            type Output = <<$wrapper as $crate::Wrapper>::Inner as ::core::ops::Index<Idx>>::Output;
            fn index(&self, idx: Idx) -> &Self::Output {
                &self.0.index(idx)
            }
        }
    };
}

/// Implement [`Display`](::core::fmt::Display) for the wrapper.
#[macro_export]
macro_rules! impl_display {
    ($wrapper:ident) => {
        impl ::core::fmt::Display for $wrapper
        where
            <$wrapper as $crate::Wrapper>::Inner: ::core::fmt::Display,
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}
