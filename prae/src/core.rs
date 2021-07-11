use core::hash::Hash;
use std::ops::{Deref, Index};
use std::{error, fmt};

/// An error occured during [construction](Guarded::new).
#[derive(Debug)]
pub struct ConstructionError<G: Guard> {
    /// Original error in case of calling [`define!`](prae_macro::define) with `validate` or just a
    /// string in case of `ensure`.
    pub inner: G::Error,
    /// The value that caused the error.
    pub value: G::Target,
}

impl<G: Guard> ConstructionError<G> {
    /// Get inner error.
    pub fn into_inner(self) -> G::Error {
        self.inner
    }
}

// FIXME: the compiler thinks that `G::Error` can be `ConstructionError<G>`,
// which conflicts with default implementation From<T> for T.
// I have no idea how to fix it!
// impl<G: Guard> From<ConstructionError<G>> for G::Error {
//     fn from(err: ConstructionError<G>) -> Self {
//         err.inner
//     }
// }

impl<G: Guard> fmt::Display for ConstructionError<G>
where
    G::Error: fmt::Debug + fmt::Display,
    G::Target: fmt::Debug,
    G: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to create {} from value {:?}: {}",
            G::alias_name(),
            self.value,
            self.inner,
        )
    }
}

impl<G: Guard> error::Error for ConstructionError<G>
where
    G::Error: fmt::Debug + fmt::Display,
    G::Target: fmt::Debug,
    G: fmt::Debug,
{
}

/// An error occured during [mutation](Guarded::try_mutate).
#[derive(Debug)]
pub struct MutationError<G: Guard> {
    /// Original error in case of calling [`define!`](prae_macro::define) with `validate` or just a
    /// string in case of `ensure`.
    pub inner: G::Error,
    /// The value before mutation.
    pub old_value: G::Target,
    /// The value that caused the error.
    pub new_value: G::Target,
}

impl<G: Guard> MutationError<G> {
    /// Get inner error.
    pub fn into_inner(self) -> G::Error {
        self.inner
    }
}

// FIXME: the compiler thinks that `G::Error` can be `MutationError<G>`,
// which conflicts with default implementation From<T> for T.
// I have no idea how to fix it!
// impl<G: Guard> From<MutationError<G>> for G::Error {
//     fn from(err: MutationError<G>) -> Self {
//         err.inner
//     }
// }

impl<G: Guard> fmt::Display for MutationError<G>
where
    G::Error: fmt::Debug + fmt::Display,
    G::Target: fmt::Debug,
    G: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to mutate {} from value {:?} to {:?}: {}",
            G::alias_name(),
            self.old_value,
            self.new_value,
            self.inner,
        )
    }
}

impl<G: Guard> error::Error for MutationError<G>
where
    G::Error: fmt::Debug + fmt::Display,
    G::Target: fmt::Debug,
    G: fmt::Debug,
{
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
    /// Helper method for useful error representation.
    fn alias_name() -> &'static str {
        std::any::type_name::<Self>()
    }
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
    pub fn new<V: Into<T>>(v: V) -> Result<Self, ConstructionError<G>> {
        let mut v: T = v.into();
        G::adjust(&mut v);
        match G::validate(&v) {
            None => Ok(Self(v)),
            Some(e) => Err(ConstructionError { inner: e, value: v }),
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
        match G::validate(&self.0) {
            None => {}
            Some(e) => panic!("validation failed with error {:?}", e),
        };
    }

    /// Mutates current value using provided closure. Will return an error if
    /// the result of the mutation is invalid.
    pub fn try_mutate(&mut self, f: impl FnOnce(&mut T)) -> Result<(), MutationError<G>>
    where
        T: Clone,
    {
        let mut cloned = self.0.clone();
        f(&mut cloned);
        G::adjust(&mut cloned);
        match G::validate(&cloned) {
            None => {
                self.0 = cloned;
                Ok(())
            }
            Some(e) => Err(MutationError {
                inner: e,
                old_value: self.0.clone(),
                new_value: cloned,
            }),
        }
    }

    /// Retrieve the inner, unprotected value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

#[cfg(feature = "unchecked-access")]
impl<T, E, G> Guarded<G>
where
    E: fmt::Debug,
    G: Guard<Target = T, Error = E>,
{
    /// Construct a value without calling `adjust` and `validate`. The invariant must be upheld
    /// manually. Should be used only for optimisation purposes.
    pub fn new_unchecked<V: Into<T>>(v: V) -> Self {
        let v: T = v.into();
        debug_assert!(G::validate(&v).is_none());
        Self(v)
    }

    /// Mutate a value without calling `adjust` and `validate`. The invariant must be upheld
    /// manually. Should be used only for optimisation purposes.
    pub fn mutate_unchecked(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.0);
        debug_assert!(G::validate(&self.0).is_none());
    }

    /// Gives mutable access to the internals without upholding invariants.
    /// They must continue to be upheld manually while the reference lives!
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }

    /// Verifies invariants. This is guaranteed to succeed unless you've used
    /// one of the `*_unchecked` methods that require variants to be manually upheld.
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
    G: fmt::Debug,
    G::Target: serde::Deserialize<'de> + std::fmt::Debug,
    G::Error: std::fmt::Display + std::fmt::Debug,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(G::Target::deserialize(deserializer)?)
            .map_err(|e: ConstructionError<G>| serde::de::Error::custom(e))
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
