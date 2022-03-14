use prae::Wrapper;

prae::define! {
    #[derive(Debug)]
    Username: String;
    plugins: [
        prae::impl_display,
    ];
}

#[test]
fn display_works() {
    let un = Username::new("lala").unwrap();
    assert_eq!(format!("{}", un), "lala");
}
