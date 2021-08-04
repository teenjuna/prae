#[cfg(feature = "unchecked")]
mod tests {
    use prae::Guard;

    prae::define! {
        pub Username: String
        ensure |u| !u.is_empty()
    }

    #[test]
    fn unchecked_construction_never_fails() {
        let u = Username::new_unchecked("lala");
        assert_eq!(u.get(), "lala");
        let u = Username::new_unchecked("");
        assert_eq!(u.get(), "");
    }

    #[test]
    fn unchecked_mutation_never_fails() {
        let mut u = Username::new_unchecked("lala");
        u.mutate_unchecked(|u| *u = "lolo".to_owned());
        assert_eq!(u.get(), "lolo");
        u.mutate_unchecked(|u| *u = "".to_owned());
        assert_eq!(u.get(), "");
    }

    #[test]
    fn get_mut_works() {
        let mut u = Username::new_unchecked("lala");
        *u.get_mut() = "lolo".to_owned();
        assert_eq!(u.get(), "lolo");
        *u.get_mut() = "".to_owned();
        assert_eq!(u.get(), "");
    }
}
