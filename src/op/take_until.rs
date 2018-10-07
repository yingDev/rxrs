use std::marker::PhantomData;
use std::boxed::FnBox;
use std::sync::Arc;
use std::sync::atomic::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use crate::*;
use crate::util::*;


pub struct OpTakeUntil<V, E, VS, ES, Src, Sig, SS: YesNo>
{
    src: Src,
    sig: Sig,
    PhantomData: PhantomData<(V, E, VS, ES, SS)>
}

pub trait ObservableOpTakeUntil<V, E, VS, ES, Sig, SS: YesNo> : Sized
{
    fn take_until(self, sig: Sig) -> OpTakeUntil<V, E, VS, ES, Self, Sig, SS>
    {
        OpTakeUntil { src:self, sig,  PhantomData }
    }
}

impl<'s, 'o, V:Clone+'o, E:Clone+'o, Src: Observable<'o, V, E>+'s, VS:Clone+'o, ES:Clone+'o, Sig: Observable<'o, VS, ES>> ObservableOpTakeUntil<V, E, VS, ES, Sig, NO> for Src where Src : Observable<'o, V, E> {}
impl<V:Clone+Send+Sync+'static, E:Clone+Send+Sync+'static, VS: Clone+Send+Sync+'static, ES:Clone+Send+Sync+'static, Src: ObservableSendSync<V, E>, Sig: ObservableSendSync<VS, ES>> ObservableOpTakeUntil<V, E, VS, ES, Sig, YES> for Src where Src : ObservableSendSync<V, E> {}


impl<'s, 'o, V:Clone+'o, E:Clone+'o, Src: Observable<'o, V, E>+'s, VS:Clone+'o, ES:Clone+'o, Sig: Observable<'o, VS, ES>> Observable<'o, V, E> for OpTakeUntil<V, E, VS, ES, Src, Sig, NO>
{
    #[inline(always)]
    fn subscribe(&self, observer: impl Observer<V, E> + 'o) -> Subscription<'o, NO>
    {
        subscribe_nss(self, Rc::new(observer))
    }
}

impl<V:Clone+Send+Sync+'static, E:Clone+Send+Sync+'static, VS: Clone+Send+Sync+'static, ES:Clone+Send+Sync+'static, Src: ObservableSendSync<V, E>, Sig: ObservableSendSync<VS, ES>> ObservableSendSync<V, E> for OpTakeUntil<V, E, VS, ES, Src, Sig, YES>
{
    #[inline(always)]
    fn subscribe(&self, observer: impl Observer<V,E>+Send+Sync+'static) -> Subscription<'static, YES>
    {
        subscribe_ss(self, Arc::new(observer))
    }
}

//todo: macro ?

#[inline(never)]
fn subscribe_nss<'s, 'o, V:Clone+'o, E:Clone+'o, Src: Observable<'o, V, E>+'s, VS:Clone+'o, ES:Clone+'o, Sig: Observable<'o, VS, ES>>
    (selv: &OpTakeUntil<V,E,VS,ES,Src,Sig,NO>, observer: Rc<Observer<V,E>+'o>) -> Subscription<'o, NO>
{
    let (sub, sub1, sub2, sub3) = Subscription::new().cloned4();
    let (o1, o2) = observer.clone2();

    let sigsub = selv.sig.subscribe((
        move |v| { if(!sub1.is_done()) { sub1.unsub(); o1.complete(); } },
        move |e| { sub2.unsub(); },
        move | | { if(!sub3.is_done()) { sub3.unsub(); o2.complete(); } }));

    if sigsub.is_done() { return sigsub; }
    sub.added_each(&sigsub).added_each(&selv.src.subscribe(observer))
}

#[inline(never)]
fn subscribe_ss<V:Clone+Send+Sync+'static, E:Clone+Send+Sync+'static, VS: Clone+Send+Sync+'static, ES:Clone+Send+Sync+'static, Src: ObservableSendSync<V, E>, Sig: ObservableSendSync<VS, ES>>
    (selv: &OpTakeUntil<V,E,VS,ES,Src,Sig,YES>, observer: Arc<Observer<V,E>+Send+Sync+'static>) -> Subscription<'static, YES>
{
    let (sub, sub1, sub2, sub3) = Subscription::new().cloned4();
    let (o1, o2) = observer.clone2();

    let sigsub = selv.sig.subscribe((
        move |v| { if(!sub1.is_done()) { sub1.unsub(); o1.complete(); } },
        move |e| { sub2.unsub(); },
        move | | { if(!sub3.is_done()) { sub3.unsub(); o2.complete(); } }));

    if sigsub.is_done() { return sigsub; }
    sub.added_each(&sigsub).added_each(&selv.src.subscribe(observer))
}

#[cfg(test)]
mod test
{
    use std::rc::Rc;
    use std::cell::Cell;
    use crate::*;

    #[test]
    fn basic()
    {
        let n = Cell::new(0);
        let c = Cell::new(0);
        let src = Rc::new(Subject::<i32, (), NO>::new());
        let sig = Rc::new(Subject::<i32, (), NO>::new());

        src.clone().take_until(sig.clone()).subscribe((
            |v| n.replace(v),
            (),
            | | c.replace(c.get() + 1))
        );

        src.next(1);
        assert_eq!(n.get(), 1);

        sig.next(0);
        src.next(2);
        assert_eq!(n.get(), 1);

        assert_eq!(c.get(), 1);
    }

    #[test]
    fn complete_before()
    {
        let n = Cell::new(0);
        let c = Cell::new(0);
        let src = Rc::new(Subject::<i32, (), NO>::new());
        let sig = Rc::new(Subject::<i32, (), NO>::new());

        sig.complete();

        src.clone().take_until(sig.clone()).subscribe((
            |v| n.replace(v),
            (),
            | | c.replace(c.get() + 1))
        );

        src.next(1);
        assert_eq!(n.get(), 0);

        sig.next(0);
        src.next(2);
        assert_eq!(n.get(), 0);

        assert_eq!(c.get(), 1);
    }
}