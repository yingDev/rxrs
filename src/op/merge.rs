use crate::*;

pub trait ObsMergeOp<'o, SS:YesNo, By: RefOrVal> : Sized
{
    fn merge(self, other: impl Observable<'o, SS, By>+'o) -> Merge<'o, 'o, SS, By>;
}

impl<'o, SS:YesNo, By: RefOrVal+'o, Src: Observable<'o, SS, By>+'o>
ObsMergeOp<'o, SS, By>
for Src
{
    fn merge(self, other: impl Observable<'o, SS, By>+'o) -> Merge<'o, 'o, SS, By>
    {
        Merge::new(vec![self.into_dyn(), other.into_dyn()])
    }
}

pub trait DynObsMergeOp<'o, SS:YesNo, By: RefOrVal> : Sized
{
    fn merge(self, other: impl Observable<'o, SS, By>+'o) -> DynObservable<'o, 'o, SS, By>;
}

impl<'o, SS:YesNo, By: RefOrVal+'o>
DynObsMergeOp<'o, SS, By>
for DynObservable<'o, 'o, SS, By>
{
    fn merge(self, other: impl Observable<'o, SS, By>+'o) -> DynObservable<'o, 'o, SS, By>
    {
        Merge::new(vec![self, other.into_dyn()]).into_dyn()
    }
}

#[cfg(test)]
mod test
{
    use crate::*;
    use std::rc::Rc;
    use std::cell::RefCell;
    use crate::util::clones::Clones;
    
    #[test]
    fn smoke()
    {
        let (n, n1, n2) = Rc::new(RefCell::new(String::new())).clones();
        
        let a = Of::value(123);
        let b = Of::<NO, i32>::value(456);
        
        a.merge(b).subscribe(|v:&_|{
            n1.borrow_mut().push_str(&format!("{}", v));
        }, |_| {
            n2.borrow_mut().push_str("ok");
        });
        
        assert_eq!("123456ok", n.borrow().as_str());
    }
}