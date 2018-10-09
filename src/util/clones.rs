
pub trait Clones<R> : Clone
{
    fn clones(self) -> R;
}

impl<T:Clone> Clones<T> for T
{
    fn clones(self) -> T
    {
        self
    }
}

impl<T:Clone> Clones<(T, T)> for T
{
    fn clones(self) -> (T, T)
    {
        (self.clone(), self)
    }
}

impl<T:Clone> Clones<(T, T, T)> for T
{
    fn clones(self) -> (T, T, T)
    {
        (self.clone(), self.clone(), self)
    }
}

impl<T:Clone> Clones<(T, T, T, T)> for T
{
    fn clones(self) -> (T, T, T, T)
    {
        (self.clone(), self.clone(), self.clone(), self)
    }
}

impl<T:Clone> Clones<(T, T, T, T, T)> for T
{
    fn clones(self) -> (T, T, T, T, T)
    {
        (self.clone(), self.clone(), self.clone(), self.clone(), self)
    }
}

impl<T:Clone> Clones<(T, T, T, T, T, T)> for T
{
    fn clones(self) -> (T, T, T, T, T, T)
    {
        (self.clone(), self.clone(), self.clone(), self.clone(), self.clone(), self)
    }
}

impl<T:Clone> Clones<(T, T, T, T, T, T, T)> for T
{
    fn clones(self) -> (T, T, T, T, T, T, T)
    {
        (self.clone(), self.clone(), self.clone(), self.clone(), self.clone(), self.clone(), self)
    }
}

pub trait RcExt
{
    type Weak;
    fn weak(&self) -> Self::Weak;
}

impl<T> RcExt for ::std::rc::Rc<T>
{
    type Weak = ::std::rc::Weak<T>;
    fn weak(&self) -> Self::Weak
    {
        ::std::rc::Rc::downgrade(self)
    }
}

impl<T> RcExt for ::std::sync::Arc<T>
{
    type Weak = ::std::sync::Weak<T>;
    fn weak(&self) -> Self::Weak
    {
        ::std::sync::Arc::downgrade(self)
    }
}

pub fn weak<R: RcExt>(rc: &R) -> R::Weak
{
    rc.weak()
}


#[cfg(test)]
mod test
{
    use std::rc::Rc;
    use super::*;

    #[test]
    fn compiles()
    {
        let x = 1;
        let (a,b) = x.clones();
        let (c,d,e) = x.clones();

        let rc = Rc::new(0);

        let mut x = 1;

        let cb = hello((|rc,b,c| ||{
            println!("ok?");
        })(rc.weak(),b,c));

        fn hello(f: impl FnOnce()){ f() }
    }
}