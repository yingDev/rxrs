<p align="center">
<img src="https://github.com/yingDev/rxrs/blob/master/assets/logo.png?raw=true">
<br>
    <b> RxRs - <a href="http://reactivex.io"> Reactive Extensions</a> for Rust </b>
</p>
<br>

### 🌱  This project is currently at its early stage... most of the features is experimental!
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
let clicks = btn_clicks(button.clone()).publish();

let sub = clicks.rx().map(|i| format!("{} Clicks", i)).sub_scoped(
    move |s:String| button.set_label(&s)
);

let sub2 = rxfac::timer(0, Some(250), GtkScheduler::get())
    .take_until(clicks.rx().skip(3))
    .map(|i| format!("{}", i))
    .sub_scoped((
        move |s:String| win1.set_title(&s),
        (),
        move | | win2.set_title("Stopped!")
    ));

clicks.connect();

gtk::main();
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
- [ ] advanced operators,factories,`Scheduler`s
- [ ] provide practical examples
- [ ] docs
- [ ] release a crate
