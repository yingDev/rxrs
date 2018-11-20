<p align="center">
<img src="https://github.com/yingDev/rxrs/blob/master/assets/logo.png?raw=true">
<br>
    <b> RxRs - <a href="http://reactivex.io"> Reactive Extensions</a> for Rust </b>
<br><br>
<a href="https://crates.io/crates/rxrs">
    <img src="https://img.shields.io/badge/crates.io-0.2.0--beta3-orange.svg">
</a>
</p>
<br>

__WIP: rxrs is under heavy development & not ready for production__


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

        evens.subscribe(
            |v: String| output.borrow_mut().push_str(&v),
            |e| output.borrow_mut().push_str("ok")
        );

        for i in 0..10 {
            subj.next(i);
        }

        assert_eq!("*0*2*4*6ok", &*output.borrow());
    }

```


```bash
src
├── by.rs
├── error.rs
├── fac
│   ├── iter.rs
│   ├── merge.rs
│   ├── mod.rs
│   ├── of.rs
│   └── timer.rs
├── lib.rs
├── observable.rs
├── observer.rs
├── op
│   ├── filter.rs
│   ├── map.rs
│   ├── merge.rs
│   ├── mod.rs
│   ├── skip.rs
│   ├── start.rs
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
│   ├── act.rs
│   ├── any_send_sync.rs
│   ├── mod.rs
│   ├── re_spin_lock.rs
│   ├── re_spin_mutex.rs
│   ├── recur_cell.rs
│   ├── ss_mark.rs
│   └── yesno.rs
├── unsub.rs
└── util
    ├── clones.rs
    └── mod.rs


```