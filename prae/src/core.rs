use std::marker::PhantomData;
use thiserror::Error;

/// A error that will be returned by [`EnsureGuarded`](EnsureGuarded) in
/// case of invalid construction or mutation.
#[derive(Debug, PartialEq, Error)]
#[error("provided value is not valid")]
pub struct ValidationError;

/// A guard that validates bounded type with a function that returns
/// `bool`.
pub trait EnsureGuard {
    /// A bounded type.
    type Target;
    /// A function that can make small adjustments of the
    /// provided value before validation.
    fn adjust(v: &mut Self::Target);
    /// A function that validates provided value. If the value
    /// is not valid, it returns `false`.
    fn ensure(v: &Self::Target) -> bool;
}

/// A thin wrapper around a type guarded by [`EnsureGuard`](EnsureGuard).
/// It is generic over inner type `T` and the `EnsureGuard` that targets it.
#[derive(Debug)]
pub struct EnsureGuarded<T, G: EnsureGuard<Target = T>>(T, PhantomData<G>);

impl<T, G> EnsureGuarded<T, G>
where
    G: EnsureGuard<Target = T>,
{
    /// Constructor. Will return an error if the provided argument `v`
    /// doesn't pass the validation.
    pub fn new<V: Into<T>>(v: V) -> Result<Self, ValidationError> {
        let mut v: T = v.into();
        G::adjust(&mut v);
        if G::ensure(&v) {
            Ok(Self(v, Default::default()))
        } else {
            Err(ValidationError {})
        }
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
        if !G::ensure(&self.0) {
            panic!("validation failed");
        };
    }

    /// Mutates current value using provided closure. Will return an error if
    /// the result of the mutation is invalid.
    pub fn try_mutate(&mut self, f: impl FnOnce(&mut T)) -> Result<(), ValidationError>
    where
        T: Clone,
    {
        let mut cloned = self.0.clone();
        f(&mut cloned);
        G::adjust(&mut cloned);
        if G::ensure(&cloned) {
            self.0 = cloned;
            Ok(())
        } else {
            Err(ValidationError)
        }
    }
}

/// A guard that validates bounded type with a function that returns
/// `Option<CustomError>`.
pub trait ValidateGuard<E> {
    /// A bounded type.
    type Target;
    /// A function that can make small adjustments of the
    /// provided value before validation.
    fn adjust(v: &mut Self::Target);
    /// A function that validates provided value. If the value
    /// is not valid, it returns `Some(CustomError)`, where `CustomError`
    /// is specified by the user.
    fn validate(v: &Self::Target) -> Option<E>;
}

/// A thin wrapper around a type guarded by [`ValidateGuard`](ValidateGuard).
/// It is generic over inner type `T`, custom error `E` and the
/// `ValidateGuard` that targets them.
#[derive(Debug)]
pub struct ValidateGuarded<T, E, G: ValidateGuard<E, Target = T>>(
    T,
    PhantomData<E>,
    PhantomData<G>,
);

impl<T, E, G> ValidateGuarded<T, E, G>
where
    E: std::fmt::Debug,
    G: ValidateGuard<E, Target = T>,
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
        impl ValidateGuard<()> for UsernameGuard {
            type Target = String;
            fn adjust(v: &mut Self::Target) {
                *v = v.trim().to_owned();
            }
            fn validate(v: &Self::Target) -> Option<()> {
                if v.is_empty() {
                    Some(())
                } else {
                    None
                }
            }
        }
        type Username = ValidateGuarded<String, (), UsernameGuard>;

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

    mod ensure_guard {
        use crate::core::*;

        #[derive(Debug)]
        struct UsernameGuard;
        impl EnsureGuard for UsernameGuard {
            type Target = String;
            fn adjust(v: &mut Self::Target) {
                *v = v.trim().to_owned();
            }
            fn ensure(v: &Self::Target) -> bool {
                !v.is_empty()
            }
        }
        type Username = EnsureGuarded<String, UsernameGuard>;

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
