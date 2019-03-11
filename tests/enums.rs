use arg_enum_proc_macro::ArgEnum;

#[derive(ArgEnum, PartialEq, Debug)]
pub enum Foo {
    Bar,
    /// Foo
    Baz
}

// should fail to compile
/*
#[derive(ArgEnum)]
pub enum Complex {
    A,
    B(Foo),
    C{a: usize, b: usize},
}
*/

#[test]
fn parse() {
    let v: Foo = "Baz".parse().unwrap();

    assert_eq!(v, Foo::Baz);
}

#[test]
fn variants() {
    assert_eq!(&Foo::variants(), &["Bar", "Baz"]);
}
