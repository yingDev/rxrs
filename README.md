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

    rx::timer(0, Some(10), NewThreadScheduler::get())
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
There's a crate [./rx-gtk](https://github.com/yingDev/rxrs/tree/master/rx-gtk) aiming to provide utilities for working with rx-gtk.Take a look at the [example_gui](https://github.com/yingDev/rxrs/tree/master/example_gui).

 <img width="300" src="https://github.com/yingDev/rxrs/blob/master/assets/eg.png?raw=true">

 ```rust
fn main()
{
    if let Err(e) = gtk::init(){ panic!("{}", e); }
    rx::scheduler::set_sync_context(Some(GtkScheduler::<rx::util::mss::No>::get()));
    load_style();

    let builder = gtk::Builder::new_from_string(include_str!("win.glade"));
    let window: gtk::Window = builder.get_object("win").unwrap();

    event_once!(window, connect_delete_event, true).subf(|_| gtk::main_quit());

    let vm = Rc::new(TodoViewModel::new());
    vm.add_item("hello world".to_owned());

    let view = TodoView::new(&builder);
    view.bind(vm.clone());

    window.show_all();
    gtk::main();
}
 ```

# File Structure
```
src
├── behaviour_subject.rs
├── behaviour_subject_nss.rs
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
│   ├── start_with.rs
│   ├── sub_on.rs
│   ├── take.rs
│   ├── take_until.rs
│   └── tap.rs
├── scheduler.rs
├── subject.rs
├── subject_nss.rs
├── subref.rs
├── test_fixture.rs
└── util
    ├── arc_cell.rs
    ├── atomic_option.rs
    ├── capture_by_clone.rs
    ├── mod.rs
    └── mss.rs

```

# TODO
- [x] basic operators,factories,`Scheduler`s
- [x] refactor towards zero-cost abstractions
- [ ] WIP: advanced operators,factories,`Scheduler`s
- [ ] WIP: provide practical examples
- [ ] docs
- [ ] release a crate
