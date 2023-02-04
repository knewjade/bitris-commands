use bitris::prelude::*;

/// This trait for direct processing without creating vec.
pub trait ForEachVisitor<T: ?Sized> {
    fn visit(&mut self, arg: &T);
}

// TODO desc
pub trait ShapeMatcher {
    fn has_next(&self) -> bool;

    fn match_shape(&self, target: Shape) -> (bool, Self);
}
