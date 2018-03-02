# rxrs - Reactive Extensions for Rust

# Work In Progress 

```
src
├── behaviour_subject.rs
├── fac
│   ├── create.rs
│   └── mod.rs
├── lib.rs
├── observable.rs
├── op
│   ├── filter.rs
│   ├── map.rs
│   ├── mod.rs
│   ├── take.rs
│   └── take_until.rs
├── subject.rs
├── subscriber.rs
├── unsub_ref.rs
└── util
    ├── arc_cell.rs
    ├── atomic_option.rs
    └── mod.rs
```

# Exapmple

```rust
#[test]
fn hello_world()
{
    let mut result = String::new();

    let src = rxfac::create(|o|
    {
        o.next("hello");
        o.next("world");
        o.complete();
        UnsubRef::empty()
    });

    src.rx().take(1).map(|s| s.to_uppercase()).sub_scoped(|s:String| result.push_str(&s));
    src.rx().skip(1).sub_scoped(|s| result.push_str(s));

    assert_eq!(result, "HELLOworld");
}
```
