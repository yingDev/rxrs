
use crate::*;

struct BehaviorSubject<'o, V:Clone+'o, E:Clone+'o, SS:YesNo>
{
    value: V,
    subj: Subject<'o, V, E, SS>
}

//impl<'o, V:Clone+'o, E:Clone+'o> Observable<'o, V, E> for BehaviorSubject<'o, V, E, NO>
//{
//    fn subscribe()
//}