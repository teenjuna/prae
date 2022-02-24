use std::borrow::Borrow;
use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::{Deref, Index};

/// A trait that describes a type, a process through which the type's value must
/// go on every construction and mutation, and a type of an error that can
/// occur during the process.
pub trait Bound {
    /// The name of the type that will utilize this bound.
    const TYPE_NAME: &'static str;

    /// The underlying type.
    type Target: fmt::Debug;

    /// The type of possible error.
    type Error: fmt::Debug;

    /// The process that will be called on every construction and mutation
    /// of the inner value.
    fn process(value: &mut Self::Target) -> Result<(), Self::Error>;
}

// TODO: add examples for methods.
/// A thin wrapper around an inner type that utilizes [`Bound`](Bound) `B`
/// to ensure the inner value is always valid.
///
/// It provides several default methods that allow you to construct, modify
/// and get the inner value.
///
/// It also implements a bunch of default traits (such as [`Clone`](Clone),
/// [`Eq`](Eq) and others) if the underlying type implements them. It
/// allows this wrapper to act as similar as possible to the underlying
/// type. For example, if your underlying type is a [`Vec`](Vec) and you
/// want to get some item by it's index, you can avoid doing `v.get()[i]`
/// and do `v[i]` directly!
#[derive(Debug)]
pub struct Bounded<T, B: Bound<Target = T>>(T, PhantomData<B>);

impl<B: Bound> Bounded<B::Target, B> {
    /// Constructs new bounded type with the provided value. Will return
    /// an error if the value doesn't hold it's invariants.
    pub fn new<V>(value: V) -> Result<Self, ConstructionError<B>>
    where
        V: Into<B::Target>,
    {
        let mut value = value.into();
        match B::process(&mut value) {
            Ok(()) => Ok(Self(value, PhantomData)),
            Err(inner) => Err(ConstructionError { inner, value }),
        }
    }

    /// Returns a shared reference to the inner value.
    pub fn get(&self) -> &B::Target {
        &self.0
    }

    /// Replaces inner value with the provided value. Will return an error
    /// if the value doesn't hold it's invariants after replacement.
    pub fn set<V>(&mut self, value: V) -> Result<(), ConstructionError<B>>
    where
        V: Into<B::Target>,
    {
        let mut value = value.into();
        match B::process(&mut value) {
            Ok(()) => {
                self.0 = value;
                Ok(())
            }
            Err(inner) => Err(ConstructionError { inner, value }),
        }
    }

    /// Mutates inner value using provided function. Will return an error
    /// if the value doesn't hold it's invariants after mutation.
    ///
    /// In order to ensure that an invalid attempt to mutate the value
    /// doesn't break it, the underlying type must be `Clone`.
    pub fn mutate<F>(&mut self, mutate: F) -> Result<(), MutationError<B>>
    where
        F: FnOnce(&mut B::Target),
        B::Target: Clone,
    {
        let mut cloned = self.0.clone();
        mutate(&mut cloned);
        match B::process(&mut cloned) {
            Ok(()) => {
                self.0 = cloned;
                Ok(())
            }
            Err(inner) => Err(MutationError {
                inner,
                old_value: self.0.clone(),
                new_value: cloned,
            }),
        }
    }

    /// Same as [`new`](Self::new), but without applying associated bound's
    /// process. This makes the caller responsible for the validity of inner
    /// value. Use only for optimization purposes.
    #[cfg(feature = "unprocessed")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unprocessed")))]
    pub fn new_unprocessed<V>(value: V) -> Self
    where
        V: Into<B::Target>,
    {
        let mut value = value.into();
        debug_assert!(B::process(&mut value).is_ok());
        Self(value, PhantomData)
    }

    /// Same as [`set`](Self::set), but without applying associated bound's
    /// process. This makes the caller responsible for the validity of inner
    /// value. Use only for optimization purposes.
    #[cfg(feature = "unprocessed")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unprocessed")))]
    pub fn set_unprocessed<V>(&mut self, value: V)
    where
        V: Into<B::Target>,
    {
        let mut value = value.into();
        debug_assert!(B::process(&mut value).is_ok());
        self.0 = value;
    }

    /// Same as [`mutate`](Self::mutate), but without applying associated
    /// bound's process and requiring underlying type to be `Clone`.
    /// This makes the caller responsible for the validity of inner value.
    /// Use only for optimization purposes.
    #[cfg(feature = "unprocessed")]
    #[cfg_attr(docsrs, doc(cfg(feature = "unprocessed")))]
    pub fn mutate_unprocessed<F>(&mut self, mutate: F)
    where
        F: FnOnce(&mut B::Target),
    {
        mutate(&mut self.0);
        debug_assert!(B::process(&mut self.0).is_ok());
    }
}

impl<T, B> Clone for Bounded<T, B>
where
    T: Clone,
    B: Bound<Target = T>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<T, B> Borrow<T> for Bounded<T, B>
where
    B: Bound<Target = T>,
{
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T, B> AsRef<T> for Bounded<T, B>
where
    B: Bound<Target = T>,
{
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T, B> Deref for Bounded<T, B>
where
    B: Bound<Target = T>,
{
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T, B> PartialEq for Bounded<T, B>
where
    T: PartialEq,
    B: Bound<Target = T>,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T, B> Eq for Bounded<T, B>
where
    T: Eq,
    B: Bound<Target = T>,
{
}

impl<T, B> PartialOrd for Bounded<T, B>
where
    T: PartialOrd,
    B: Bound<Target = T>,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T, B> Ord for Bounded<T, B>
where
    T: Ord,
    B: Bound<Target = T>,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T, B> Copy for Bounded<T, B>
where
    T: Copy,
    B: Bound<Target = T>,
{
}

impl<T, B> Hash for Bounded<T, B>
where
    T: Hash,
    B: Bound<Target = T>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<U, T, B> Index<U> for Bounded<T, B>
where
    T: Index<U>,
    B: Bound<Target = T>,
{
    type Output = T::Output;
    fn index(&self, index: U) -> &Self::Output {
        self.0.index(index)
    }
}

// FIXME: conflicts with blanket impl.
// impl<T, B> TryFrom<T> for Bounded<T, B>
// where
//     B: Bound<Target = T>,
// {
//     type Error = ConstructionError<B>;
//     fn try_from(value: T) -> Result<Self, Self::Error> {
//         Self::new(value)
//     }
// }

/// An error that occurs when bounded type is constructed with a value that
/// isn't valid. It contains some inner error that describes the reason for the
/// error as well as the value that caused the error.
#[derive(Debug)]
pub struct ConstructionError<B: Bound> {
    pub inner: B::Error,
    pub value: B::Target,
}

impl<B> fmt::Display for ConstructionError<B>
where
    B: Bound,
    B::Error: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to construct type {} from value {:?}: {}",
            B::TYPE_NAME,
            self.value,
            self.inner,
        )
    }
}

impl<B> Error for ConstructionError<B>
where
    B: Bound + fmt::Debug,
    B::Error: fmt::Display,
{
    // NOTE: `self.inner` could be used for `source` function.
    // However, it would require `B::Error: Error + 'static`,
    // which is more restrictive, therefore less appealing.
    // It's also not clear for me if this change would be
    // useful.
    // Waiting for the stabilization of specialization?
}

/// An error that occurs when bounded type is mutated in a way so that it's
/// inner value isn't valid anymore. It contains some inner error that describes
/// the reason for the error as well as both the value before mutation and the
/// value after mutation (that caused the error).
#[derive(Debug)]
pub struct MutationError<B: Bound> {
    pub inner: B::Error,
    pub old_value: B::Target,
    pub new_value: B::Target,
}

impl<B> fmt::Display for MutationError<B>
where
    B: Bound,
    B::Error: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "failed to mutate type {} from value {:?} to value {:?}: {}",
            B::TYPE_NAME,
            self.old_value,
            self.new_value,
            self.inner,
        )
    }
}

impl<B> Error for MutationError<B>
where
    B: Bound + fmt::Debug,
    B::Error: fmt::Display,
{
    // NOTE: `self.inner` could be used for `source` function.
    // However, it would require `B::Error: Error + 'static`,
    // which is more restrictive, and therefore less appealing.
    // It's also not clear for me if this change would be
    // useful.
    // Waiting for the stabilization of specialization?
}

/// Convenience trait that allows mapping from `Result<_,
/// ConstructionError<Bound>>` and `Result<_, MutationError<Bound>` to
/// `Result<_, Bound::Error>`.
pub trait MapInnerError<O, E> {
    fn map_inner(self) -> Result<O, E>;
}

impl<O, B> MapInnerError<O, B::Error> for Result<O, ConstructionError<B>>
where
    B: Bound,
{
    fn map_inner(self) -> Result<O, B::Error> {
        self.map_err(|err| err.inner)
    }
}

impl<O, B> MapInnerError<O, B::Error> for Result<O, MutationError<B>>
where
    B: Bound,
{
    fn map_inner(self) -> Result<O, B::Error> {
        self.map_err(|err| err.inner)
    }
}

#[cfg(feature = "serde")]
impl<'de, T, B> serde::Deserialize<'de> for Bounded<T, B>
where
    T: serde::Deserialize<'de> + std::fmt::Debug,
    B: Bound<Target = T> + fmt::Debug,
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
impl<T, B> serde::Serialize for Bounded<T, B>
where
    T: serde::Serialize,
    B: Bound<Target = T>,
    B::Error: std::fmt::Display + std::fmt::Debug,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        B::Target::serialize(self.get(), serializer)
    }
}
