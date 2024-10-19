use stakker::{Ret, CX};

pub trait ActorIterator<T>: Sized {
    fn next(&mut self, cx: CX![], callback: Ret<T>);
}

