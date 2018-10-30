<p align="center">
<img src="https://github.com/yingDev/rxrs/blob/master/assets/logo.png?raw=true">
<br>
    <b> RxRs - <a href="http://reactivex.io"> Reactive Extensions</a> for Rust </b>
<br><br>
<a href="https://crates.io/crates/rxrs">
    <img src="https://img.shields.io/badge/crates.io-0.1.0--alpha4-orange.svg">
</a>
</p>
<br>

### 🌱 WIP: rewriting everything ...


```rust
    use rxrs::*;

    #[test]
    pub fn greet()
    {
        let output = RefCell::new(String::new());

        let subj = Rc::new(Subject::<NO, i32>::new());

        let evens: impl Observable<NO, Val<String>> = subj.clone()
            .filter(|v:&_| v%2 == 0 )
            .take(4)
            .map(|v:&_| format!("*{}", v));

        evens.sub(
            |v: String| output.borrow_mut().push_str(&v),
            |e: Option<&_>| output.borrow_mut().push_str("ok")
        );

        for i in 0..10 {
            subj.next(i);
        }

        assert_eq!("*0*2*4*6ok", &*output.borrow());
    }

```


```bash
src
├── act.rs
├── act_helpers.rs
├── fac
│   ├── mod.rs
│   ├── of.rs
│   └── timer.rs
├── lib.rs
├── observables.rs
├── op
│   ├── filter.rs
│   ├── map.rs
│   ├── mod.rs
│   ├── take.rs
│   └── until.rs
├── scheduler
│   ├── current_thread_scheduler.rs
│   ├── event_loop_scheduler.rs
│   ├── mod.rs
│   └── new_thread_scheduler.rs
├── subject
│   ├── behavior_subject.rs
│   ├── mod.rs
│   └── subject.rs
├── sync
│   ├── mod.rs
│   └── re_spin_lock.rs
├── unsub.rs
└── util
    ├── alias.rs
    ├── any_send_sync.rs
    ├── by.rs
    ├── clones.rs
    ├── mod.rs
    └── yesno.rs


```