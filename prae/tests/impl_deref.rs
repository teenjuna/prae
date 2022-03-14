use prae::Wrapper;

prae::define! {
    #[derive(Clone, Debug)]
    ImplementsClone: String;
    plugins: [
        prae::impl_deref,
    ];
}

prae::define! {
    #[derive(Debug)]
    NotImplementsClone: String;
    plugins: [
        prae::impl_deref,
    ];
}

#[test]
#[allow(clippy::redundant_clone)]
fn deref_works() {
    let ic = ImplementsClone::new("lala").unwrap();
    let ic_clone = ic.clone(); // implemented Clone at work
    assert_eq!(ic_clone.get(), "lala");

    let nic = NotImplementsClone::new("lala").unwrap();
    let nic_clone = nic.clone(); // Deref at work
    assert_eq!(nic_clone, "lala");
}
