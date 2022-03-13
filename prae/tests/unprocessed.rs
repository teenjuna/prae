#[cfg(feature = "unprocessed")]
mod tests {
    use prae::Wrapper;

    prae::define! {
        pub Username: String;
        ensure |u| !u.is_empty();
    }

    #[test]
    fn unprocessed_construction_never_fails() {
        let u = Username::new_unprocessed("lala");
        assert_eq!(u.get(), "lala");
        let u = Username::new_unprocessed("");
        assert_eq!(u.get(), "");
    }

    #[test]
    fn unprocessed_mutation_never_fails() {
        let mut u = Username::new_unprocessed("lala");
        u.mutate_unprocessed(|u| *u = "lolo".to_owned());
        assert_eq!(u.get(), "lolo");
        u.mutate_unprocessed(|u| *u = "".to_owned());
        assert_eq!(u.get(), "");
    }
}
