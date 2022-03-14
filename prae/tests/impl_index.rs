use prae::Wrapper;

prae::define! {
    #[derive(Debug)]
    Numbers: Vec<u64>;
    plugins: [
        prae::impl_index,
    ];
}

#[test]
fn index_works() {
    let nums = Numbers::new([1, 2, 3]).unwrap();
    assert_eq!(nums[1], 2);
}
