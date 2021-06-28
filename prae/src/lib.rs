mod core;

pub use crate::core::*;
pub use prae_macro::define;

#[cfg(test)]
mod tests {
    mod adjust_ensure {
        use crate as prae;

        prae::define! {
            pub Username: String;
            adjust |u| *u = u.trim().to_owned();
            ensure |u| !u.is_empty();
        }

        #[test]
        fn construction_fails_for_invalid_data() {
            assert_eq!(Username::new("  ").unwrap_err(), prae::ValidationError {});
        }

        #[test]
        fn construction_succeeds_for_valid_data() {
            let un = Username::new(" user ").unwrap();
            assert_eq!(un.get(), "user");
        }

        #[test]
        fn mutation_fails_for_invalid_data() {
            let mut un = Username::new("user").unwrap();
            assert_eq!(
                un.try_mutate(|u| *u = "  ".to_owned()).unwrap_err(),
                prae::ValidationError {}
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
    }

    mod adjust_validate {
        use crate as prae;

        #[derive(Debug, PartialEq)]
        pub struct UsernameError;

        prae::define! {
            pub Username: String;
            adjust   |u| *u = u.trim().to_owned();
            validate |u| -> Option<UsernameError> {
                if u.is_empty() {
                    Some(UsernameError)
                } else {
                    None
                }
            };
        }

        #[test]
        fn construction_fails_for_invalid_data() {
            assert_eq!(Username::new("  ").unwrap_err(), UsernameError {});
        }

        #[test]
        fn construction_succeeds_for_valid_data() {
            let un = Username::new(" user ").unwrap();
            assert_eq!(un.get(), "user");
        }

        #[test]
        fn mutation_fails_for_invalid_data() {
            let mut un = Username::new("user").unwrap();
            assert_eq!(
                un.try_mutate(|u| *u = "  ".to_owned()).unwrap_err(),
                UsernameError {}
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
    }
}
