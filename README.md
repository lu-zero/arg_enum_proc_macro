# Procedural macro derive that mimics `arg_enum!` from [clap](https://clap.rs)

## Usage

In `Cargo.toml`:
``` toml
[dependencies]
arg_enum_proc_macro = "0.1"
```

In the rust code:
``` rust
use arg_enum_proc_macro::ArgEnum;

/// All the possible states of Foo
#[derive(ArgEnum)]
pub enum Foo {
    /// Initial state
    Unk,
    /// Foo is on
    On,
    /// Foo is off
    Off,
}
```
