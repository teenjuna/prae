use prae::Guard;

prae::define! {
    Text: String
    adjust |t| *t = t.trim().to_owned()
    validate |t| -> Result<(), &'static str> {
        if t.is_empty() {
            Err("provided text is empty")
        } else {
            Ok(())
        }
    }
}

prae::extend! {
    CapText: Text
    adjust |t| {
       let mut cs = t.chars();
        *t = cs.next().unwrap().to_uppercase().collect::<String>() + cs.as_str();
    }
}

prae::extend! {
    Sentence: CapText
    validate |s| -> Result<(), String> {
        eprintln!("{}", &s);
        if s.ends_with(&['.', '!', '?'][..]) {
            Ok(())
        } else {
            Err("provided sentence has no ending punctuation mark".to_owned())
        }
    }
}

#[test]
fn extended_works() {
    let t = CapText::new("   a couple of words").unwrap();
    assert_eq!(t.get(), "A couple of words");

    let e = CapText::new(" ").unwrap_err();
    assert_eq!(e.into_inner(), "provided text is empty");
}

#[test]
fn double_extended_works() {
    let t = Sentence::new(" a sentence. ").unwrap();
    assert_eq!(t.get(), "A sentence.");

    let e = Sentence::new(" ").unwrap_err();
    assert_eq!(e.into_inner(), "provided text is empty");

    let e = Sentence::new(" a sentence ").unwrap_err();
    assert_eq!(
        e.into_inner(),
        "provided sentence has no ending punctuation mark"
    );
}
