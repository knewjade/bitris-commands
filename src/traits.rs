/// This trait for direct processing without creating vec.
pub trait ForEachVisitor<T: ?Sized> {
    fn visit(&mut self, arg: &T);
}
