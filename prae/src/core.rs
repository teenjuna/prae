use std::marker::PhantomData;

pub trait Guard<E> {
    type Target;
    fn adjust(v: &mut Self::Target);
    fn validate(v: &Self::Target) -> Option<E>;
}

#[derive(Debug)]
pub struct Guarded<T, E, G: Guard<E, Target = T>>(T, PhantomData<E>, PhantomData<G>);

impl<T, E, G> Guarded<T, E, G>
where
    E: std::fmt::Debug,
    G: Guard<E, Target = T>,
{
    pub fn new<V: Into<T>>(v: V) -> Result<Self, E> {
        let mut v: T = v.into();
        G::adjust(&mut v);
        G::validate(&v).map_or(Ok(()), Err)?;
        Ok(Self(v, Default::default(), Default::default()))
    }

    pub fn get(&self) -> &T {
        &self.0
    }

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
    use super::*;

    #[derive(Debug)]
    struct UsernameGuard;
    impl Guard<()> for UsernameGuard {
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
    type Username = Guarded<String, (), UsernameGuard>;

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
    }
}
