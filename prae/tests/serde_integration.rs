#[cfg(feature = "serde")]
mod tests {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct User {
        name: Username,
    }

    prae::define! {
        Username: String
        adjust |u| *u = u.trim().to_owned()
        ensure |u| !u.is_empty()
    }

    #[test]
    fn deserialization_succeeds_with_valid_data() {
        let json = r#"
        {
            "name": "   some name   "
        }
        "#;
        let u: User = serde_json::from_str(json).unwrap();
        assert_eq!(u.name.get(), "some name");
    }

    #[test]
    fn deserialization_fails_with_invalid_data() {
        let json = r#"
        {
            "name": "     "
        }
        "#;
        let err = serde_json::from_str::<User>(json).unwrap_err();
        assert_eq!(err.to_string(), "value is invalid at line 4 column 9");
    }

    #[test]
    fn serialization_succeeds() {
        let u = User {
            name: Username::new("some name").unwrap(),
        };
        let json = serde_json::to_string(&u).unwrap();
        assert_eq!(r#"{"name":"some name"}"#, json)
    }
}
