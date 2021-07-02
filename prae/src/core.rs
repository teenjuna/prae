use std::marker::PhantomData;
use thiserror::Error;

/// Default validation error. It is used for [`define!`](prae_macro::define) macro with `ensure`
/// keyword.
#[derive(Debug, PartialEq, Error)]
#[error("provided value is not valid")]
pub struct ValidationError;

/// A trait that represents a guard bound, e.g. a type that is being guarded, `adjust`/`validate`
/// functions and a possible validation error.
pub trait Guard {
    /// The type that is being guarded.
    type Target;
    /// An error that will be returned in case of failed validation.
    type Error;
    /// A function that can make small adjustments of the
    /// provided value before validation.
    fn adjust(v: &mut Self::Target);
    /// A function that validates provided value. If the value
    /// is not valid, it returns `Some(Self::Error)`.
    fn validate(v: &Self::Target) -> Option<Self::Error>;
}

/// A thin wrapper around an underlying type and a [`Guard`](Guard) bounded to it. It guarantees
/// to always hold specified invariants and act as close as possible to the underlying type.
#[derive(Debug)]
pub struct Guarded<T, E, G: Guard<Target = T, Error = E>>(T, PhantomData<E>, PhantomData<G>);

impl<T, E, G> Guarded<T, E, G>
where
    E: std::fmt::Debug,
    G: Guard<Target = T, Error = E>,
{
    /// Constructor. Will return an error if the provided argument `v`
    /// doesn't pass the validation.
    pub fn new<V: Into<T>>(v: V) -> Result<Self, E> {
        let mut v: T = v.into();
        G::adjust(&mut v);
        G::validate(&v).map_or(Ok(()), Err)?;
        Ok(Self(v, Default::default(), Default::default()))
    }

    /// Returns a shared reference to the inner value.
    pub fn get(&self) -> &T {
        &self.0
    }

    /// Mutates current value using provided closure. Will panic if
    /// the result of the mutation is invalid.
    pub fn mutate(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.0);
        G::adjust(&mut self.0);
        // We have to match here because Option.expect_none is unstable.
        // See: https://github.com/rust-lang/rust/issues/62633
        match G::validate(&self.0) {
            None => {}
            Some(e) => panic!("validation failed with error {:?}", e),
        };
    }

    /// Mutates current value using provided closure. Will return an error if
    /// the result of the mutation is invalid.
    pub fn try_mutate(&mut self, f: impl FnOnce(&mut T)) -> Result<(), E>
    where
        T: Clone,
    {
        let mut cloned = self.0.clone();
        f(&mut cloned);
        G::adjust(&mut cloned);
        G::validate(&cloned).map_or(Ok(()), Err)?;
        self.0 = cloned;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    mod validate_guard {
        use crate::core::*;

        #[derive(Debug)]
        struct UsernameGuard;
        impl Guard for UsernameGuard {
            type Target = String;
            type Error = &'static str;
            fn adjust(v: &mut Self::Target) {
                *v = v.trim().to_owned();
            }
            fn validate(v: &Self::Target) -> Option<Self::Error> {
                if v.is_empty() {
                    Some("username is empty")
                } else {
                    None
                }
            }
        }
        type Username = Guarded<String, &'static str, UsernameGuard>;

        #[test]
        fn construction_with_valid_value_succeeds() {
            let un = Username::new(" username\n").unwrap();
            assert_eq!(un.get(), "username");
        }

        #[test]
        fn construction_with_invalid_value_fails() {
            Username::new("   \n").unwrap_err();
        }

        #[test]
        fn mutation_with_valid_value_succeeds() {
            let mut un = Username::new("username").unwrap();
            un.mutate(|v| *v = format!(" new {}\n", v));
            assert_eq!(un.get(), "new username");
        }

        #[test]
        #[should_panic]
        fn mutation_with_invalid_value_panics() {
            let mut un = Username::new("username").unwrap();
            un.mutate(|v| *v = "   \n".to_owned());
        }

        #[test]
        fn falliable_mutation_with_valid_value_succeds() {
            let mut un = Username::new("username").unwrap();
            un.try_mutate(|v| *v = format!(" new {}\n", v)).unwrap();
            assert_eq!(un.get(), "new username");
        }

        #[test]
        fn falliable_mutation_with_valid_value_fails() {
            let mut un = Username::new("username").unwrap();
            un.try_mutate(|v| *v = "   \n".to_owned()).unwrap_err();
            assert_eq!(un.get(), "username");
        }
    }
}
