use bitris::prelude::Shape;

/// Represents that shape is undetermined.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub(crate) enum FuzzyShape {
    #[default] Unknown,
    Known(Shape),
}
