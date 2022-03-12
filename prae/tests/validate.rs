use assert_matches::assert_matches;
use prae::{MapInnerError, Wrapper};

#[derive(Debug, PartialEq, Eq)]
pub struct UsernameError;

impl std::fmt::Display for UsernameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "username is empty")
    }
}

prae::define! {
    #[derive(Debug)]
    pub Username: String;
    validate(UsernameError) |u| {
        if u.is_empty() {
            Err(UsernameError{})
        } else {
            Ok(())
        }
    };
}

#[test]
fn construction_error_formats_correctly() {
    let err = Username::new("").unwrap_err();
    assert_eq!(
        err.to_string(),
        "failed to construct type Username from value \"\": username is empty"
    );
}

#[test]
fn mutation_error_formats_correctly() {
    let mut un = Username::new("user").unwrap();
    let err = un.mutate(|u| *u = "".to_owned()).unwrap_err();
    assert_eq!(
        err.to_string(),
        "failed to mutate type Username from value \"user\" to value \"\": username is empty"
    );
}

#[test]
fn construction_error_can_be_transormed_into_inner() {
    let _err = || -> Result<(), UsernameError> {
        Username::new("").map_inner()?;
        Ok(())
    }();
}

#[test]
fn mutation_error_can_be_transormed_into_inner() {
    let _err = || -> Result<(), UsernameError> {
        let mut un = Username::new("user").unwrap();
        un.mutate(|u| *u = "".to_owned()).map_inner()?;
        Ok(())
    }();
}

#[test]
fn construction_fails_for_invalid_data() {
    assert_matches!(
        Username::new(""),
        Err(prae::ConstructionError { inner, .. }) if inner == UsernameError {}
    );
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
        un.mutate(|u| *u = "".to_owned()),
        Err(prae::MutationError { inner, .. }) if inner == UsernameError {}
    );
}

#[test]
fn mutation_succeeds_for_valid_data() {
    let mut un = Username::new("user").unwrap();
    un.mutate(|u| *u = " new user ".to_owned()).unwrap();
    assert_eq!(un.get(), " new user ");
}
