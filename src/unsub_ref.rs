use std::sync::Once;
use std::sync::ONCE_INIT;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::collections::LinkedList;
use util::AtomicOption;

use util::ArcCell;
use std::marker::PhantomData;

static mut EMPTY_SUB_REF: Option<UnsubRef> = None;
static EMPTY_SUB_REF_INIT: Once = ONCE_INIT;

#[derive(Clone)]
pub struct UnsubRef<'a>
{
    state: Arc<State<'a>>,
    _unsub_on_drop: bool,
}

impl<'a> Drop for UnsubRef<'a>
{
    fn drop(&mut self)
    {
        if self._unsub_on_drop {
            self.unsub();
        }
    }
}

struct State<'a>
{
    disposed: AtomicBool,
    cb: Box<Fn()+'a+Send+Sync>,
    extra: AtomicOption<LinkedList<UnsubRef<'a>>>,
}

impl<'a> UnsubRef<'a>
{
    pub fn fromFn<F: Fn()+'a+Send+Sync>(unsub: F) -> UnsubRef<'a>
    {
         UnsubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: box unsub,
                extra: AtomicOption::with(LinkedList::new())
            }),
            _unsub_on_drop: false,
        }
    }

    pub fn signal() -> UnsubRef<'static>
    {
        UnsubRef::fromFn(||{})
    }

    pub fn scoped<'s>() -> UnsubRef<'s>
    {
        UnsubRef { state: Arc::new(
            State{
                disposed: AtomicBool::new(false),
                cb: box ||{},
                extra: AtomicOption::with(LinkedList::new())
            }),
            _unsub_on_drop: true,
        }
    }

    pub fn empty() -> UnsubRef<'a>
    {
        unsafe {
            EMPTY_SUB_REF_INIT.call_once(|| {
                //todo
                EMPTY_SUB_REF = Some( UnsubRef {
                    state: Arc::new(
                    State{cb: box ||{}, extra: AtomicOption::new(), disposed: AtomicBool::new(true) },
                ),
                    _unsub_on_drop: false,
                } );
            });
            ::std::mem::transmute(EMPTY_SUB_REF.as_ref().unwrap().clone())
        }
    }

    pub fn unsub(&self)
    {
        if self.state.disposed.compare_and_swap(false, true, Ordering::SeqCst){ return; }
        (self.state.cb)();

        loop{
            if let Some(lst) = self.state.extra.take(Ordering::SeqCst){
                lst.iter().for_each(|f| f.unsub());
                break;
            }
        }

    }

    pub fn add<U>(&self, unsub: U) -> &Self where U:IntoUnsubRef<'a>
    {
        if self.state.disposed.load(Ordering::SeqCst) {
            unsub.into().unsub();
        }else {
            let un = unsub.into();
            if un.disposed() { return self; }

            loop{
                if let Some(mut lst) = self.state.extra.take(Ordering::SeqCst){
                    if ! un.disposed() {
                        lst.push_back(un);
                    }
                    self.state.extra.swap(lst, Ordering::SeqCst);
                    break;
                }
            }

        }
        return self;
    }

    pub fn combine<U>(self, other: U) -> Self where U : IntoUnsubRef<'a>
    {
        self.add(other.into());
        self
    }

    #[inline]
    pub fn disposed(&self) -> bool { self.state.disposed.load(Ordering::SeqCst)}
}

pub trait IntoUnsubRef<'a>
{
    fn into(self) -> UnsubRef<'a>;
}
impl<'a, F:Fn()+'static+Send+Sync> IntoUnsubRef<'a> for F
{
    fn into(self) -> UnsubRef<'a>
    {
        UnsubRef::fromFn(self)
    }
}
impl<'a> IntoUnsubRef<'a> for UnsubRef<'a>
{
    fn into(self) -> UnsubRef<'a>{ self }
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::thread;
    use subject::*;
    use observable::*;

    #[test]
    fn disposed()
    {
        let u = UnsubRef::fromFn(||{});
        u.unsub();
        assert!(u.disposed());
    }

    #[test]
    fn add_after_unsub()
    {
        let u = UnsubRef::fromFn(||{});
        u.unsub();

        let v = Arc::new(AtomicBool::new(false));
        let v2 = v.clone();
        u.add(move || v2.store(true, Ordering::SeqCst));

        assert!(v.load(Ordering::SeqCst));
    }

    #[test]
    fn threads()
    {
        let out = Arc::new(Mutex::new("".to_owned()));
        let out2 = out.clone();
        let out3 = out.clone();

        let u = UnsubRef::fromFn(||{});
        let u2 = u.clone();
        let u3 = u.clone();

        let j = thread::spawn(move || {
            u2.add(move || out.lock().unwrap().push_str("A"));
        });
        u3.add(move || out2.lock().unwrap().push_str("A"));

        let j2  = thread::spawn(move || u.unsub());

        j.join();
        j2.join();

        assert_eq!(*out3.lock().unwrap(), "AA");

    }

    #[test]
    fn scoped()
    {
        let subj = Subject::new();
        let mut result = 0;
        {
            let scope = subj.rx().sub_scoped(|v|{ result += v; });
            scope.add(|| println!("Where?"));
            println!("hello");
            subj.next(1);
        }
        subj.next(2);

        assert_eq!(result, 1);
    }

    #[test]
    fn scoped2()
    {
        use fac::rxfac;
        use op::*;

        let mut result = 0;
        rxfac::range(0..100).take(10).sub_scoped(|v| result += v );

        assert_eq!(result, (0..100).take(10).sum());
    }
}