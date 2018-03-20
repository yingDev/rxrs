<p align="center">
<img src="https://github.com/yingDev/rxrs/blob/master/assets/logo.png?raw=true">
<br>
    <b> RxRs - <a href="http://reactivex.io"> Reactive Extensions</a> for Rust </b>
</p>
<br>

### 🌱  This project is currently at its early stage... most of the features are experimental!
### 🦀  Contributions Are Welcome!

# Example
### Basics
```rust
(Rust Nightly 1.25+)

#[test]
fn timer()
{
    println!("cur thread {:?}", thread::current().id());

    rxfac::timer(0, Some(10), NewThreadScheduler::get())
        .skip(3)
        .filter(|i| i % 2 == 0)
        .take(3)
        .map(|v| format!("-{}-", v))
        .observe_on(NewThreadScheduler::get())
        .subf(
            |v| println!("{} on {:?}", v, thread::current().id()),
            (),
            | | println!("complete on {:?}", thread::current().id())
        );

    thread::sleep(::std::time::Duration::from_millis(2000));
}
```
Output:
```bash
cur thread ThreadId(1)
-4- on ThreadId(2)
-6- on ThreadId(2)
-8- on ThreadId(2)
complete on ThreadId(2)
```

### Play with [gtk-rs](https://github.com/gtk-rs/gtk) 
```rust 
fn main()
{
    if gtk::init().is_err() { return; }

    let window = Window::new(WindowType::Toplevel);
    window.connect_delete_event(|_, _| { gtk::main_quit(); Inhibit(false) });

    let btn = Button::new_with_label("Click me!");
    window.add(&btn);

    let clicks = rxrs::create_boxed(|o| {
        let i = Cell::new(0);
        let id = btn.connect_clicked(move |_| o.next(i.replace(i.get() + 1)) );
    });

    let btn = btn.clone();
    clicks.subf(move |i| btn.set_label( &format!("hello {}", i) ) );

    window.show_all();
    gtk::main();
}
```
<img width="200" src="https://github.com/yingDev/rxrs/blob/master/assets/gtk.gif?raw=true">

# File Structure
```
src
├── behaviour_subject.rs
├── connectable_observable.rs
├── fac
│   ├── create.rs
│   ├── mod.rs
│   └── timer.rs
├── lib.rs
├── observable.rs
├── op
│   ├── concat.rs
│   ├── debounce.rs
│   ├── filter.rs
│   ├── map.rs
│   ├── mod.rs
│   ├── multicast.rs
│   ├── observe_on.rs
│   ├── publish.rs
│   ├── skip.rs
│   ├── sub_on.rs
│   ├── take.rs
│   ├── take_until.rs
│   └── tap.rs
├── scheduler.rs
├── subject.rs
├── subscriber.rs
├── unsub_ref.rs
└── util
    ├── arc_cell.rs
    ├── atomic_option.rs
    └── mod.rs

```

# TODO
- [x] basic operators,factories,`Scheduler`s
- [x] refactor towards zero-cost abstractions
- [ ] WIP: advanced operators,factories,`Scheduler`s
- [ ] WIP: provide practical examples
- [ ] docs
- [ ] release a crate
