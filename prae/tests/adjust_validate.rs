use assert_matches::assert_matches;
use prae::Wrapper;

#[derive(Debug)]
pub struct UsernameError;

prae::define! {
    #[derive(Debug)]
    pub Username: String;
    adjust   |u| *u = u.trim().to_owned();
    validate(UsernameError) |u| {
        if u.is_empty() {
            Err(UsernameError)
        } else {
            Ok(())
        }
    };
}

#[test]
fn construction_fails_for_invalid_data() {
    assert_matches!(Username::new("  "), Err(prae::ConstructionError { .. }));
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
        un.mutate(|u| *u = "  ".to_owned()),
        Err(prae::MutationError { .. })
    );
}

#[test]
fn mutation_succeeds_for_valid_data() {
    let mut un = Username::new("user").unwrap();
    un.mutate(|u| *u = "  new user  ".to_owned()).unwrap();
    assert_eq!(un.get(), "new user");
}
