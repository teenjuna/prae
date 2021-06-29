//! This crate provides a way to define type wrappers that behave as close as possible to
//! the underlying type, but guarantee to uphold arbitrary invariants at all times.
//!
//! # Examples
//! Let's create a `Username` type. It will be a wrapper around non-empty `String`:
//! ```
//! prae::define!(pub Username: String ensure |u| !u.is_empty());
//!
//! // We can't create an invalid username.
//! assert!(Username::new("").is_err());
//!
//! // But we can create a valid type!
//! let mut u = Username::new("valid name").unwrap();
//! assert_eq!(u.get(), "valid name");
//!
//! // We can mutate it:
//! assert!(u.try_mutate(|u| *u = "new name".to_owned()).is_ok());
//! assert_eq!(u.get(), "new name"); // our name has changed!
//!
//! // But we can't make it invalid:
//! assert!(u.try_mutate(|u| *u = "".to_owned()).is_err());
//! assert_eq!(u.get(), "new name"); // our name hasn't changed!
//!
//! // Let's try this...
//! assert!(Username::new("  ").is_ok()); // looks kind of invalid though :(
//! ```
//! As you can see, the last example treats `"  "` as a valid username, but it's not. We
//! can of course do something like `Username::new(s.trim())` every time, but why should
//! we do it ourselves? Let's automate it!
//! ```
//! prae::define! {
//!     pub Username: String
//!     adjust |u| *u = u.trim().to_string()
//!     ensure |u| !u.is_empty()
//! }
//!
//! let mut u = Username::new(" valid name \n\n").unwrap();
//! assert_eq!(u.get(), "valid name"); // now we're talking!
//!
//! // This also works for mutations:
//! assert!(matches!(u.try_mutate(|u| *u = "   ".to_owned()), Err(prae::ValidationError)));
//! ```
//! Now our `Username` trims provided value automatically.
//!
//! You might noticed that `prae::ValidationError` is returned by default when our
//! construction/mutation fails. Altough it's convenient, there are situations when you might
//! want to return a custom error. And `prae` can help with this:
//! ```
//! #[derive(Debug)]
//! struct UsernameError;
//!
//! prae::define! {
//!     pub Username: String
//!     adjust   |u| *u = u.trim().to_string()
//!     validate |u| -> Option<UsernameError> {
//!         if u.is_empty() {
//!             Some(UsernameError)
//!         } else {
//!             None
//!         }
//!     }
//! }
//!
//! assert!(matches!(Username::new("  "), Err(UsernameError)));
//! ```

mod core;

pub use crate::core::*;
pub use prae_macro::define;

#[cfg(test)]
mod tests {
    mod ensure {
        use crate as prae;

        prae::define! {
            pub Username: String
            ensure |u| !u.is_empty()
        }

        #[test]
        fn construction_fails_for_invalid_data() {
            assert_eq!(Username::new("").unwrap_err(), prae::ValidationError);
        }

        #[test]
        fn construction_succeeds_for_valid_data() {
            let un = Username::new(" user ").unwrap();
            assert_eq!(un.get(), " user ");
        }

        #[test]
        fn mutation_fails_for_invalid_data() {
            let mut un = Username::new("user").unwrap();
            assert_eq!(
                un.try_mutate(|u| *u = "".to_owned()).unwrap_err(),
                prae::ValidationError
            );
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
    }

    mod adjust_and_ensure {
        use crate as prae;

        prae::define! {
            pub Username: String
            adjust |u| *u = u.trim().to_owned()
            ensure |u| !u.is_empty()
        }

        #[test]
        fn construction_fails_for_invalid_data() {
            assert_eq!(Username::new("  ").unwrap_err(), prae::ValidationError);
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
                prae::ValidationError
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

    mod validate {
        use crate as prae;

        #[derive(Debug, PartialEq)]
        pub struct UsernameError;

        prae::define! {
            pub Username: String
            validate |u| -> Option<UsernameError> {
                if u.is_empty() {
                    Some(UsernameError{})
                } else {
                    None
                }
            }
        }

        #[test]
        fn construction_fails_for_invalid_data() {
            assert_eq!(Username::new("").unwrap_err(), UsernameError {});
        }

        #[test]
        fn construction_succeeds_for_valid_data() {
            let un = Username::new(" user ").unwrap();
            assert_eq!(un.get(), " user ");
        }

        #[test]
        fn mutation_fails_for_invalid_data() {
            let mut un = Username::new("user").unwrap();
            assert_eq!(
                un.try_mutate(|u| *u = "".to_owned()).unwrap_err(),
                UsernameError {}
            );
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
    }

    mod adjust_and_validate {
        use crate as prae;

        #[derive(Debug, PartialEq)]
        pub struct UsernameError;

        prae::define! {
            pub Username: String
            adjust   |u| *u = u.trim().to_owned()
            validate |u| -> Option<UsernameError> {
                if u.is_empty() {
                    Some(UsernameError)
                } else {
                    None
                }
            }
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
