use prae::Wrapper;

prae::define! {
    #[derive(Debug)]
    Username: String;
    adjust |u| *u = u.trim().to_owned();
}

#[test]
fn construction_adjusted() {
    let u = Username::new("  ").unwrap();
    assert_eq!(u.get(), "");
}

#[test]
fn mutation_adjusted() {
    let mut u = Username::new("something").unwrap();
    u.mutate(|u| *u = "  something new   ".to_owned()).unwrap();
    assert_eq!(u.get(), "something new");
}
