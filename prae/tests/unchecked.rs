#[cfg(feature = "unchecked-access")]
mod tests {
    use assert_matches::assert_matches;

    use prae;

    prae::define! {
        pub Username: String
        ensure |u| !u.is_empty()
    }

    #[test]
    #[should_panic]
    fn construction_fails_for_invalid_data_unchecked_succeeds() {
        assert_matches!(
            Username::new("").unwrap_err(),
            prae::ConstructionError { .. }
        );

        Username::new_unchecked("");
    }

    #[test]
    #[should_panic]
    fn mutation_fails_for_invalid_data_unchecked_succeeds() {
        let mut un = Username::new("user").unwrap();
        let err = un.try_mutate(|u| *u = "".to_owned()).unwrap_err();
        assert_matches!(err, prae::MutationError { .. });

        un.mutate_unchecked(|u| *u = "".to_owned());
    }

    #[test]
    fn mutation_fails_for_invalid_data_get_mut_succeeds() {
        let mut un = Username::new("user").unwrap();
        let err = un.try_mutate(|u| *u = "".to_owned()).unwrap_err();
        assert_matches!(err, prae::MutationError { .. });

        let t = un.get_mut();
        *t = "".to_owned();
        assert_eq!(un.get(), "");
    }
}
