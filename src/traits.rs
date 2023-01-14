/// This trait for direct processing without creating vec.
pub trait ForEachVisitor<T: ?Sized> {
    fn visit(&mut self, arg: &T);
}

/// This trait creates new data by binding data.
/// The data created through this function will have references and other associations to the generator.
///
/// Therefore, the lifetime is taken over at the same time.
/// If the creation fails, an error may be returned.
pub trait TryBind<'a, T> {
    type Error;

    fn try_bind(&'a self) -> Result<T, Self::Error>;
}
