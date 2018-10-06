pub trait CollectionExt<V>
{
    fn replace_all<I: Iterator<Item=V>>(&mut self, items: I) -> &mut Self;
}

impl<V> CollectionExt<V> for Vec<V>
{
    fn replace_all<I:Iterator<Item=V>>(&mut self, items: I) -> &mut Self
    {
        self.clear();
        self.extend(items);
        return self;
    }
}