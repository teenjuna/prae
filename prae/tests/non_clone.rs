struct User {
    name: String,
}

prae::define! {
    ValidUser: User;
    ensure |u| !u.name.is_empty();
}
