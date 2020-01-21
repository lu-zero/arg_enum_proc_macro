use arg_enum_proc_macro::ArgEnum;

#[derive(ArgEnum, PartialEq, Debug)]
pub enum Foo {
    Bar,
    /// Foo
    Baz,
}


#[test]
fn parse() {
    let v: Foo = "Baz".parse().unwrap();

    assert_eq!(v, Foo::Baz);
}

#[test]
fn variants() {
    assert_eq!(&Foo::variants(), &["Bar", "Baz"]);
}

mod alias {
    use arg_enum_proc_macro::ArgEnum;

    #[derive(ArgEnum, PartialEq, Debug)]
    pub enum Bar {
        A,
        B,
        #[arg_enum(alias = "Cat")]
        C,
    }

    #[test]
    fn parse() {
        let v: Bar = "Cat".parse().unwrap();

        assert_eq!(v, Bar::C);
    }

    #[test]
    fn variants() {
        assert_eq!(&Bar::variants(), &["A", "B", "C", "Cat"]);
    }
}

mod ui {
    #[test]
    fn invalid_applications() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/complex-enum.rs");
        t.compile_fail("tests/ui/derive-not-on-enum.rs");
    }
}
