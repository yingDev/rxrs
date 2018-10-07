use std::marker::PhantomData;
use std::boxed::FnBox;
use std::sync::Arc;
use std::sync::atomic::*;
use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;
use crate::*;
use crate::util::{CloneN, trait_alias::CSS};

pub struct OpTakeUntil<V, E, VS, ES, Src, Sig, SS: YesNo>
{
    src: Src,
    sig: Sig,
    PhantomData: PhantomData<(V, E, VS, ES, SS)>
}

pub trait ObservableOpTakeUntil<V, E, VS, ES, Sig, SS: YesNo> : Sized
{
    #[inline(always)]
    fn take_until(self, sig: Sig) -> OpTakeUntil<V, E, VS, ES, Self, Sig, SS> { OpTakeUntil { src:self, sig,  PhantomData } }
}

impl<'s, 'o, V:Clone+'o, E:Clone+'o, Src: Observable<'o,V,E>+'s, VS:Clone+'o, ES:Clone+'o, Sig: Observable<'o, VS, ES>> ObservableOpTakeUntil<V, E, VS, ES, Sig, NO> for Src {}
impl<V:CSS, E:CSS, VS: CSS, ES:CSS, Src: ObservableSendSync<V, E>, Sig: ObservableSendSync<VS, ES>> ObservableOpTakeUntil<V, E, VS, ES, Sig, YES> for Src {}


impl<'s, 'o, V:Clone+'o, E:Clone+'o, Src: Observable<'o,V,E>+'s, VS:Clone+'o, ES:Clone+'o, Sig: Observable<'o, VS, ES>> Observable<'o, V, E> for OpTakeUntil<V, E, VS, ES, Src, Sig, NO>
{
    #[inline(always)]
    fn subscribe(&self, observer: impl Observer<V, E> + 'o) -> Subscription<'o, NO>
    {
        subscribe_nss(self, Rc::new(observer))
    }
}

impl<V:CSS, E:CSS, VS:CSS, ES:CSS, Src: ObservableSendSync<V, E>, Sig: ObservableSendSync<VS, ES>> ObservableSendSync<V, E> for OpTakeUntil<V, E, VS, ES, Src, Sig, YES>
{
    #[inline(always)]
    fn subscribe(&self, observer: impl Observer<V,E>+Send+Sync+'static) -> Subscription<'static, YES>
    {
        subscribe_ss(self, Arc::new(observer))
    }
}

struct Notifier<'o, V, E, VS, ES, Dest, SS: YesNo>
{
    sub: Subscription<'o, SS>,
    dest: Dest,
    PhantomData: PhantomData<(V, E, VS, ES)>
}

impl<'o, V:Clone+'o, E:Clone+'o, VS:Clone+'o, ES:Clone+'o> Observer<VS, ES> for Notifier<'o, V, E, VS, ES, Rc<Observer<V,E>+'o>, NO>
{
    fn next(&self, v: VS) { self.sub.unsub_then(|| self.dest.complete()); }
    fn error(&self, e: ES) { self.sub.unsub(); }
    fn complete(&self) { self.sub.unsub_then(|| self.dest.complete()); }
}

impl<V:Clone+'static, E:Clone+'static, VS:Clone+'static, ES:Clone+'static> Observer<VS, ES> for Notifier<'static, V, E, VS, ES, Arc<Observer<V,E>+Send+Sync+'static>, YES>
{
    fn next(&self, v: VS) { self.sub.unsub_then(|| self.dest.complete()); }
    fn error(&self, e: ES) { self.sub.unsub(); }
    fn complete(&self) { self.sub.unsub_then(|| self.dest.complete()); }
}


#[inline(never)]
fn subscribe_nss<'s, 'o, V:Clone+'o, E:Clone+'o, Src: Observable<'o,V,E>+'s, VS:Clone+'o, ES:Clone+'o, Sig: Observable<'o, VS, ES>>
    (selv: &OpTakeUntil<V,E,VS,ES,Src,Sig,NO>, observer: Rc<Observer<V,E>+'o>) -> Subscription<'o, NO>
{
    let sub = Subscription::new();
    let sigsub = selv.sig.subscribe(Notifier{ sub: sub.clone(), dest: observer.clone(), PhantomData });
    if sigsub.is_done() { return sigsub; }
    sub.added_each(&sigsub).added_each(&selv.src.subscribe(observer))
}

#[inline(never)]
fn subscribe_ss<V:CSS, E:CSS, VS:CSS, ES:CSS, Src: ObservableSendSync<V, E>, Sig: ObservableSendSync<VS, ES>>
    (selv: &OpTakeUntil<V,E,VS,ES,Src,Sig,YES>, observer: Arc<Observer<V,E>+Send+Sync+'static>) -> Subscription<'static, YES>
{
    let sub = Subscription::new();
    let sigsub = selv.sig.subscribe(Notifier{ sub: sub.clone(), dest: observer.clone(), PhantomData });
    if sigsub.is_done() { return sigsub; }
    sub.added_each(&sigsub).added_each(&selv.src.subscribe(observer))
}

#[cfg(test)]
mod test
{
    use std::rc::Rc;
    use std::cell::Cell;
    use crate::*;
    use crate::util::CloneN;

    #[test]
    fn basic()
    {
        let n = Cell::new(0);
        let c = Cell::new(0);
        let src = Rc::new(Subject::<i32, (), NO>::new());
        let sig = Rc::new(Subject::<i32, (), NO>::new());

        src.clone().take_until(sig.clone()).subscribe((|v| n.replace(v), (), | | c.replace(c.get() + 1)));

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

        src.clone().take_until(sig.clone()).subscribe((|v| n.replace(v), (), | | c.replace(c.get() + 1)));

        src.next(1);
        assert_eq!(n.get(), 0);

        sig.next(0);
        src.next(2);
        assert_eq!(n.get(), 0);

        assert_eq!(c.get(), 1);
    }

    #[test]
    fn recursive()
    {
        let n = Cell::new(0);
        let (s, s2, s3) = Rc::new(Subject::<i32, (), NO>::new()).cloned3();

        s2.take_until(s3).subscribe((|v| n.replace(v),(), ||{ println!("complete??")}));

        s.next(1);

        assert_eq!(n.get(), 0);
    }
}