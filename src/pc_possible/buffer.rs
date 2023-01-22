use bitris::prelude::*;

#[derive(Clone, Debug)]
pub(crate) struct Buffer {
    shapes: Vec<Shape>,
    index: usize,
}

impl Buffer {
    pub(crate) fn with_resized(size: usize) -> Self {
        Self { shapes: vec![Shape::T; size], index: 0 }
    }

    pub(crate) fn increment(&mut self, shape: Shape) {
        self.shapes[self.index] = shape;
        self.index += 1;
    }

    pub(crate) fn decrement(&mut self) {
        self.index -= 1;
    }

    pub(crate) fn as_slice(&self) -> &[Shape] {
        &self.shapes[0..self.index]
    }
}
