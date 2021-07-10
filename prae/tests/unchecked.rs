use prae;

prae::define! {
    pub Username: String
    ensure |u| !u.is_empty()
}

#[test]
#[cfg(feature = "unchecked-access")]
fn construction_fails_for_invalid_data_unchecked_succeeds() {
    use assert_matches::assert_matches;
    use prae::UncheckedAccess;

    assert_matches!(
        Username::new("").unwrap_err(),
        prae::ValidationError { .. }
    );

    let un = Username::new_unchecked("").unwrap();
    assert_eq!(un.get(), " user ");
}

#[test]
#[cfg(feature = "unchecked-access")]
fn mutation_fails_for_invalid_data_unchecked_succeeds() {
    use assert_matches::assert_matches;
    use prae::UncheckedAccess;

    let mut un = Username::new("user").unwrap();
    assert_matches!(
        un.try_mutate(|u| *u = "".to_owned()).unwrap_err(),
        prae::ValidationError { .. }
    );

    un.mutate_unchecked("").unwrap();
    assert_eq!(un.get(), "");
}

#[test]
#[cfg(feature = "unchecked-access")]
fn mutation_fails_for_invalid_data_get_mut_succeeds() {
    use assert_matches::assert_matches;
    use prae::UncheckedAccess;

    let mut un = Username::new("user").unwrap();
    assert_matches!(
        un.try_mutate(|u| *u = "".to_owned()).unwrap_err(),
        prae::ValidationError { .. }
    );

    let t = un.get_mut();
    t = "";
    assert_eq!(un.get(), "");
}