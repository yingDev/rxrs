use crate::*;
use std::sync::Mutex;

pub fn iter_clone<It: Iterator+Clone>(it: It) -> Iter<It>
{
    Iter{ it }
}

pub fn iter_once<It: Iterator>(it: It) -> Iter<Wrap<It>>
{
    Iter{ it: Wrap(Mutex::new(Some(it))) }
}

pub struct Iter<It>
{
    it: It,
}

pub struct Wrap<It>(Mutex<Option<It>>);

impl<'o, It: Iterator+'o>
Observable<'o, NO, Val<It::Item>>
for Iter<Wrap<It>>
{
    fn subscribe(&self, next: impl ActNext<'o, NO, Val<It::Item>>, ec: impl ActEc<'o, NO>+'o) -> Unsub<'o, NO> where Self: Sized
    {
        let it = self.it.0.lock().unwrap().take();

        if let Some(it) = it {
            for v in it {
                if next.stopped() { break; }
                next.call(v);
            }
            ec.call_once(None);
        } else {
            ec.call_once(Some(RxError::simple(None, "Iter has been consumed")));
        }

        Unsub::done()
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, NO, Val<It::Item>>>, ec: Box<ActEcBox<'o,NO>>) -> Unsub<'o, NO>
    { self.subscribe(next, ec) }
}

impl<'o, It: Iterator+'o + Clone>
Observable<'o, NO, Val<It::Item>>
for Iter<It>
{
    fn subscribe(&self, next: impl ActNext<'o, NO, Val<It::Item>>, ec: impl ActEc<'o, NO>+'o) -> Unsub<'o, NO> where Self: Sized
    {
        for v in self.it.clone() {
            if next.stopped() { break; }
            next.call(v);
        }

        ec.call_once(None);

        Unsub::done()
    }

    fn subscribe_dyn(&self, next: Box<ActNext<'o, NO, Val<It::Item>>>, ec: Box<ActEcBox<'o,NO>>) -> Unsub<'o, NO>
    { self.subscribe(next, ec) }
}

#[cfg(test)]
mod test
{
    use crate::*;
    use std::cell::Cell;

    #[test]
    fn once()
    {
        let vec = vec![1,2,3];
        let obs = iter_once(vec.into_iter());

        let n = Cell::new(0);
        obs.subscribe(|i|{ n.replace(i); }, |e: Option<RxError>|{ assert!(e.is_none()); });
        assert_eq!(n.get(), 3);

        n.replace(0);
        obs.subscribe(|i|{ n.replace(i); }, |e: Option<RxError>| {
            assert!(e.is_some());
            e.unwrap().set_handled();
        });
        assert_eq!(n.get(), 0);
    }

    #[test]
    fn clone()
    {
        let vec = vec![1,2,3];
        let obs = iter_clone(vec.iter());

        let n = Cell::new(0);
        obs.subscribe(|i:&_| n.replace(*i), |_e| n.replace(n.get()+100) );
        assert_eq!(n.get(), 103);

        n.replace(0);
        obs.subscribe(|i:&_|{ n.replace(*i); }, |e: Option<RxError>|{
            assert!(e.is_none());
            n.replace(n.get()+100);
        });
        assert_eq!(n.get(), 103);
    }
}