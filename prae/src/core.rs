use core::hash::Hash;
use std::fmt;
use std::ops::{Deref, Index};

/// Default validation error. It is used for [`define!`](prae_macro::define) macro with `ensure`
/// keyword.
#[derive(Clone)]
pub struct ValidationError {
    /// The name of the type where this ValidationError originated.
    pub source_type: &'static str,
    /// The human readable message.
    pub message: &'static str,
}

/// Commonly used ValidationErrors
impl ValidationError {
    /// The input given was empty (whitespace-only strings are considered empty)
    pub fn new<T>(message: &'static str) -> Self {
        ValidationError {
            source_type: std::any::type_name::<T>(),
            message,
        }
    }
}

impl std::error::Error for ValidationError {}

impl fmt::Debug for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "provided value is not valid")
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "provided value is not valid")
    }
}

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

/// A thin wrapper around the underlying type and the [`Guard`](Guard) bounded to it. It guarantees
/// to always hold specified invariants and act as close as possible to the underlying type.
#[derive(Debug)]
pub struct Guarded<G: Guard>(G::Target);

impl<T, E, G> Guarded<G>
where
    E: fmt::Debug,
    G: Guard<Target = T, Error = E>,
{
    /// Constructor. Will return an error if the provided argument `v`
    /// doesn't pass the validation.
    pub fn new<V: Into<T>>(v: V) -> Result<Self, E> {
        let mut v: T = v.into();
        G::adjust(&mut v);
        G::validate(&v).map_or(Ok(()), Err)?;
        Ok(Self(v))
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

    /// Retrieve the inner, unprotected value.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Construct a value without calling `adjust` and `validate`. The invariant must be upheld
    /// manually. Should be used only for optimisation purposes.
    pub unsafe fn new_manual<V: Into<T>>(v: V) -> Self {
        let v: T = v.into();
        debug_assert!(G::validate(&v).is_none());
        Self(v)
    }

    /// Mutate a value without calling `adjust` and `validate`. The invariant must be upheld
    /// manually. Should be used only for optimisation purposes.
    pub unsafe fn mutate_manual(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.0);
        debug_assert!(G::validate(&self.0).is_none());
    }

    /// Gives mutable access to the internals without upholding invariants.
    /// They must continue to be upheld manually while the reference lives!
    pub unsafe fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Verifies invariants. This is guaranteed to succeed unless you've used
    /// one of the `unsafe` methods that require variants to be manually upheld.
    pub fn verify(&self) -> Result<(), E> {
        G::validate(&self.0).map_or(Ok(()), Err)
    }
}

impl<G: Guard> Clone for Guarded<G>
where
    G::Target: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

// impl<G: Guard> Borrow<G::Target> for Guarded<G> {
//     fn borrow(&self) -> &G::Target {
//         &self.0
//     }
// }

impl<G: Guard> AsRef<G::Target> for Guarded<G> {
    fn as_ref(&self) -> &G::Target {
        &self.0
    }
}

impl<G: Guard> Deref for Guarded<G> {
    type Target = G::Target;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<G: Guard> PartialEq for Guarded<G>
where
    G::Target: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<G: Guard> Eq for Guarded<G> where G::Target: Eq {}

impl<G: Guard> PartialOrd for Guarded<G>
where
    G::Target: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<G: Guard> Ord for Guarded<G>
where
    G::Target: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<G: Guard> Copy for Guarded<G> where G::Target: Copy {}

impl<G: Guard> Hash for Guarded<G>
where
    G::Target: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<U, G: Guard> Index<U> for Guarded<G>
where
    G::Target: Index<U>,
{
    type Output = <G::Target as Index<U>>::Output;
    fn index(&self, index: U) -> &Self::Output {
        self.0.index(index)
    }
}

#[cfg(feature = "serde")]
impl<'de, G: Guard> serde::Deserialize<'de> for Guarded<G>
where
    G::Target: serde::Deserialize<'de>,
    G::Error: std::fmt::Display + std::fmt::Debug,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(G::Target::deserialize(deserializer)?)
            .map_err(|e: G::Error| serde::de::Error::custom(e))
    }
}

#[cfg(feature = "serde")]
impl<G: Guard> serde::Serialize for Guarded<G>
where
    G::Target: serde::Serialize,
    G::Error: std::fmt::Display + std::fmt::Debug,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        G::Target::serialize(self.get(), serializer)
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
        type Username = Guarded<UsernameGuard>;

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

        #[cfg(feature = "serde")]
        mod serde {
            use super::*;
            use ::serde::{Deserialize, Serialize};

            #[derive(Debug, Deserialize, Serialize)]
            struct User {
                username: Username,
            }

            #[test]
            fn serialization_succeeds() {
                let u = User {
                    username: Username::new("  john doe  ").unwrap(),
                };
                let j = serde_json::to_string(&u).unwrap();
                assert_eq!(j, r#"{"username":"john doe"}"#)
            }

            #[test]
            fn deserialization_fails_with_invalid_value() {
                let e = serde_json::from_str::<User>(r#"{ "username": "  " }"#).unwrap_err();
                assert_eq!(e.to_string(), "username is empty at line 1 column 20");
            }

            #[test]
            fn deserialization_succeeds_with_valid_value() {
                let u = serde_json::from_str::<User>(r#"{ "username": "  john doe  " }"#).unwrap();
                assert_eq!(u.username.get(), "john doe");
            }
        }
    }
}
