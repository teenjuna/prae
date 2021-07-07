use assert_matches::assert_matches;

use prae;

prae::define! {
    pub Username: String
    adjust |u| *u = u.trim().to_owned()
    ensure |u| !u.is_empty()
}

#[test]
fn construction_fails_for_invalid_data() {
    assert_matches!(Username::new("  ").unwrap_err(), prae::ValidationError { .. });
}

#[test]
fn construction_succeeds_for_valid_data() {
    let un = Username::new(" user ").unwrap();
    assert_eq!(un.get(), "user");
}

#[test]
fn mutation_fails_for_invalid_data() {
    let mut un = Username::new("user").unwrap();
    assert_matches!(
        un.try_mutate(|u| *u = "  ".to_owned()).unwrap_err(),
        prae::ValidationError { .. }
    );
}

#[test]
#[should_panic]
fn mutation_panics_for_invalid_data() {
    let mut un = Username::new("user").unwrap();
    un.mutate(|u| *u = "  ".to_owned());
}

#[test]
fn mutation_succeeds_for_valid_data() {
    let mut un = Username::new("user").unwrap();
    assert!(un.try_mutate(|u| *u = "  new user  ".to_owned()).is_ok());
    assert_eq!(un.get(), "new user");
}
