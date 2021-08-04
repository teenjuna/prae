use core::hash::Hash;
use std::ops::{Deref, Index};
use std::{error, fmt};

/// A trait that describes behaviour of some guard wrapper. It specifies the
/// type of the inner value, the type of a possible error and the function
/// that will be called on construction and all subsequent mutations of the
/// guard wrapper.
pub trait Bound {
    /// The type of inner value.
    type Target: fmt::Debug;

    /// The type of possible validation error.
    type Error: fmt::Debug;

    /// A function that either adjusts the value, validates it, or both.
    fn apply(v: &mut Self::Target) -> Result<(), Self::Error>;
}

/// A type that has some inner value and a [`Bound`](Bound) attached to it.
/// It allows user to construct, mutate and get it's inner value, and promises
/// that inner value will be always valid according to the associated bound.
pub trait Guard
where
    Self: Sized,
{
    /// The bound of the guard.
    type Bound: Bound;

    /// Constructs the guard with the provided value. Will return an error if
    /// the value is not valid.
    fn new<V: Into<<Self::Bound as Bound>::Target>>(v: V) -> Result<Self, ConstructionError<Self>>;

    /// Returns a shared reference to the inner value.
    fn get(&self) -> &<Self::Bound as Bound>::Target;

    /// Mutates inner value using provided closure. **Will panic** if the value
    /// becomes invalid.
    fn mutate(&mut self, f: impl FnOnce(&mut <Self::Bound as Bound>::Target));

    /// Mutates inner value using provided closure. Will return an error if the
    /// value becomes instalid.
    fn try_mutate(
        &mut self,
        f: impl FnOnce(&mut <Self::Bound as Bound>::Target),
    ) -> Result<(), MutationError<Self>>
    where
        <Self::Bound as Bound>::Target: Clone;

    /// Retrieve the inner, unprotected value.
    fn into_inner(self) -> <Self::Bound as Bound>::Target;

    /// Constructs the guard with the provided value without adjusting and
    /// validating it, **making the caller responsible for the validity** of the
    /// data. **Should be used only for optimisation purposes**.
    #[cfg(feature = "unchecked")]
    fn new_unchecked<V: Into<<Self::Bound as Bound>::Target>>(v: V) -> Self;

    /// Mutates inner value using provided closure without adjustment and validation of the
    /// resulting data, **making the caller responsible for the validity** of the
    /// data. **Should be used only for optimisation purposes**.
    #[cfg(feature = "unchecked")]
    fn mutate_unchecked(&mut self, f: impl FnOnce(&mut <Self::Bound as Bound>::Target));

    /// Gives mutable access to the inner value, **making the caller responsible for the
    /// validity** of the data. **Should be used only for optimisation purposes**.
    #[cfg(feature = "unchecked")]
    fn get_mut(&mut self) -> &mut <Self::Bound as Bound>::Target;
}

/// Default guard wrapper.
#[derive(Debug)]
pub struct Guarded<B: Bound>(B::Target);

impl<T, E, B> Guard for Guarded<B>
where
    B: Bound<Target = T, Error = E>,
    E: fmt::Debug,
{
    type Bound = B;

    fn new<V: Into<T>>(v: V) -> Result<Self, ConstructionError<Self>> {
        let mut v = v.into();
        match B::apply(&mut v) {
            Ok(_) => Ok(Self(v)),
            Err(e) => Err(ConstructionError { inner: e, value: v }),
        }
    }

    fn get(&self) -> &T {
        &self.0
    }

    fn mutate(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.0);
        if let Err(e) = B::apply(&mut self.0) {
            panic!("mutation failed: {:?}", e);
        }
    }

    fn try_mutate(&mut self, f: impl FnOnce(&mut T)) -> Result<(), MutationError<Self>>
    where
        T: Clone,
    {
        let mut tmp = self.0.clone();
        f(&mut tmp);
        match B::apply(&mut tmp) {
            Ok(_) => {
                self.0 = tmp;
                Ok(())
            }
            Err(e) => Err(MutationError {
                inner: e,
                old_value: self.0.clone(),
                new_value: tmp,
            }),
        }
    }

    fn into_inner(self) -> T {
        self.0
    }

    #[cfg(feature = "unchecked")]
    fn new_unchecked<V: Into<T>>(v: V) -> Self {
        Self(v.into())
    }

    #[cfg(feature = "unchecked")]
    fn mutate_unchecked(&mut self, f: impl FnOnce(&mut T)) {
        f(&mut self.0);
    }

    #[cfg(feature = "unchecked")]
    fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

/// An error that occurs when [construction](Guard::new) of guard fails.
#[derive(Debug)]
pub struct ConstructionError<G: Guard> {
    /// The error returned by the the [`Bound::apply`](Bound::apply).
    pub inner: <G::Bound as Bound>::Error,
    /// The value that caused the error.
    pub value: <G::Bound as Bound>::Target,
}

impl<G: Guard> ConstructionError<G> {
    /// Get inner error.
    pub fn into_inner(self) -> <G::Bound as Bound>::Error {
        self.inner
    }
}

impl<G> fmt::Display for ConstructionError<G>
where
    G: Guard,
    <G::Bound as Bound>::Error: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to create {} from value {:?}: {}",
            std::any::type_name::<G>(),
            self.value,
            self.inner,
        )
    }
}

impl<G> error::Error for ConstructionError<G>
where
    G: Guard + fmt::Debug,
    G::Bound: fmt::Debug,
    <G::Bound as Bound>::Error: fmt::Display,
{
}

/// An error that occurs when [mutation](Guarded::try_mutate) of guard fails.
#[derive(Debug)]
pub struct MutationError<G: Guard> {
    /// The error returned by the the [`Bound::apply`](Bound::apply).
    pub inner: <G::Bound as Bound>::Error,
    /// The value before mutation.
    pub old_value: <G::Bound as Bound>::Target,
    /// The value that caused the error.
    pub new_value: <G::Bound as Bound>::Target,
}

impl<G: Guard> MutationError<G> {
    /// Get inner error.
    pub fn into_inner(self) -> <G::Bound as Bound>::Error {
        self.inner
    }
}

impl<G> fmt::Display for MutationError<G>
where
    G: Guard,
    <G::Bound as Bound>::Error: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to mutate {} from value {:?} to {:?}: {}",
            std::any::type_name::<G>(),
            self.old_value,
            self.new_value,
            self.inner,
        )
    }
}

impl<G> error::Error for MutationError<G>
where
    G: Guard + fmt::Debug,
    G::Bound: fmt::Debug,
    <G::Bound as Bound>::Error: fmt::Display,
{
}

impl<B> Clone for Guarded<B>
where
    B: Bound,
    B::Target: Clone,
{
    fn clone(&self) -> Self {
        todo!()
    }
}

impl<B: Bound> AsRef<B::Target> for Guarded<B> {
    fn as_ref(&self) -> &B::Target {
        &self.0
    }
}

impl<B: Bound> Deref for Guarded<B> {
    type Target = B::Target;
    fn deref(&self) -> &B::Target {
        &self.0
    }
}

impl<B> PartialEq for Guarded<B>
where
    B: Bound,
    B::Target: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<B: Bound> Eq for Guarded<B> where B::Target: Eq {}

impl<B> PartialOrd for Guarded<B>
where
    B: Bound,
    B::Target: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<B> Ord for Guarded<B>
where
    B: Bound,
    B::Target: Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<B: Bound> Copy for Guarded<B> where B::Target: Copy {}

impl<B> Hash for Guarded<B>
where
    B: Bound,
    B::Target: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<U, B> Index<U> for Guarded<B>
where
    B: Bound,
    B::Target: Index<U>,
{
    type Output = <B::Target as Index<U>>::Output;
    fn index(&self, index: U) -> &Self::Output {
        self.0.index(index)
    }
}

#[cfg(feature = "serde")]
impl<'de, B> serde::Deserialize<'de> for Guarded<B>
where
    B: Bound + fmt::Debug,
    B::Target: serde::Deserialize<'de> + std::fmt::Debug,
    B::Error: std::fmt::Display + std::fmt::Debug,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(B::Target::deserialize(deserializer)?)
            .map_err(|e| serde::de::Error::custom(e.inner))
    }
}

#[cfg(feature = "serde")]
impl<B> serde::Serialize for Guarded<B>
where
    B: Bound,
    B::Target: serde::Serialize,
    B::Error: std::fmt::Display + std::fmt::Debug,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        B::Target::serialize(self.get(), serializer)
    }
}
