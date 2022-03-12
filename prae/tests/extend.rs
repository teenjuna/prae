use prae::Wrapper;

prae::define! {
    #[derive(Debug)]
    Text: String;
    adjust |t| *t = t.trim().to_owned();
    validate(&'static str) |t| {
        if t.is_empty() {
            Err("provided text is empty")
        } else {
            Ok(())
        }
    };
}

prae::extend! {
    #[derive(Debug)]
    CapText: Text;
    adjust |t| {
       let mut cs = t.chars();
       *t = cs.next().unwrap().to_uppercase().collect::<String>() + cs.as_str();
    };
}

prae::extend! {
    #[derive(Debug)]
    Sentence: CapText;
    validate(String) |s| {
        if s.ends_with(&['.', '!', '?'][..]) {
            Ok(())
        } else {
            Err("provided sentence has no ending punctuation mark".to_owned())
        }
    };
}

#[test]
fn extended_works() {
    let t = CapText::new("   a couple of words").unwrap();
    assert_eq!(t.get(), "A couple of words");

    let e = CapText::new(" ").unwrap_err();
    assert_eq!(e.inner, "provided text is empty");
}

#[test]
fn double_extended_works() {
    let t = Sentence::new(" a sentence. ").unwrap();
    assert_eq!(t.get(), "A sentence.");

    let e = Sentence::new(" ").unwrap_err();
    assert_eq!(e.inner, "provided text is empty");

    let e = Sentence::new(" a sentence ").unwrap_err();
    assert_eq!(e.inner, "provided sentence has no ending punctuation mark");
}
