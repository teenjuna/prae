use assert_matches::assert_matches;

use prae;

prae::define! {
    pub Username: String
    ensure |u| !u.is_empty()
}

#[test]
fn construction_error_formats_correctly() {
    let err = Username::new("").unwrap_err();
    assert_eq!(
        err.to_string(),
        "failed to create Username from value \"\": provided value is invalid"
    );
}

#[test]
fn mutation_error_formats_correctly() {
    let mut un = Username::new("user").unwrap();
    let err = un.try_mutate(|u| *u = "".to_owned()).unwrap_err();
    assert_eq!(
        err.to_string(),
        "failed to mutate Username from value \"user\" to \"\": provided value is invalid"
    );
}

#[test]
fn construction_fails_for_invalid_data() {
    assert_matches!(Username::new(""), Err(prae::ConstructionError { .. }));
}

#[test]
fn construction_succeeds_for_valid_data() {
    let un = Username::new(" user ").unwrap();
    assert_eq!(un.get(), " user ");
}

#[test]
fn mutation_fails_for_invalid_data() {
    let mut un = Username::new("user").unwrap();
    assert_matches!(
        un.try_mutate(|u| *u = "".to_owned()),
        Err(prae::MutationError { .. })
    )
}

#[test]
#[should_panic]
fn mutation_panics_for_invalid_data() {
    let mut un = Username::new("user").unwrap();
    un.mutate(|u| *u = "".to_owned());
}

#[test]
fn mutation_succeeds_for_valid_data() {
    let mut un = Username::new("user").unwrap();
    assert!(un.try_mutate(|u| *u = " new user ".to_owned()).is_ok());
    assert_eq!(un.get(), " new user ");
}
