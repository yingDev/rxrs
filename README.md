<p align="center">
<img src="https://github.com/yingDev/rxrs/blob/master/assets/logo.png?raw=true">
<br>
    <b> RxRs - <a href="http://reactivex.io"> Reactive Extensions</a> for Rust </b>
</p>
<br>

[![version](https://img.shields.io/badge/crates.io-0.1.0--alpha4-orange.svg)](https://crates.io/crates/rxrs)
### 🌱 WIP: rewriting everything ...


```rust
    #[test]
    fn ops()
    {
        let timer: impl Observable<YES, Val<usize>> = Timer::new(Duration::from_millis(10), NewThreadScheduler::new(Arc::new(DefaultThreadFac)));

        let (out, out1, out3) = Arc::new(Mutex::new(String::new())).clones();

        timer.filter(|v: &_| v % 2 == 0 ).take(5).map(|v| format!("{}", v)).sub(
            move |v: String| { out.lock().unwrap().push_str(&*v); },
            move |e: Option<&_>| out3.lock().unwrap().push_str("ok")
        );

        ::std::thread::sleep_ms(1000);

        assert_eq!(out1.lock().unwrap().as_str(), "02468ok");
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