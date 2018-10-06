use std::slice::Iter;

pub(crate) struct ImmutableList<T: Clone>
{
    vec: Vec<T>
}

impl<T: Clone> ImmutableList<T>
{
    pub fn empty(init: impl FnOnce(&mut Vec<T>)) -> ImmutableList<T>
    {
        let mut vec = Vec::new();
        init(&mut vec);
        ImmutableList { vec }
    }

    pub fn len(&self) -> usize { self.vec.len() }

    pub fn add(&self, value: T) -> ImmutableList<T>
    {
        self.with(|vec: &mut Vec<T>| vec.push(value))
    }

    pub fn rm(&self, at: usize) -> ImmutableList<T>
    {
        self.with(|vec: &mut Vec<T>| { vec.remove(at); })
    }

    pub fn with(&self, cb:impl FnOnce(&mut Vec<T>)) -> ImmutableList<T>
    {
        let mut vec = self.vec.clone();
        cb(&mut vec);
        ImmutableList{ vec }
    }

    pub fn clear(&self) -> ImmutableList<T>
    {
        self.with(|vec: &mut Vec<T>| vec.clear() )
    }
}

impl<'a, T: Clone> IntoIterator for &'a ImmutableList<T>
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter
    {
        self.vec.iter()
    }
}

#[cfg(test)]
mod test
{
    #[test]
    fn hello()
    {

    }
}