extern crate core;

pub use bit_shapes::*;
pub use clipped_board::*;
pub use shape_sequence::*;
pub use patterns::*;
pub use shape_order::*;
pub use shape_counter::*;
pub use traits::*;

pub mod prelude {
    pub use bitris::prelude::*;

    pub use crate::{
        bit_shapes::*,
        clipped_board::*,
        shape_sequence::*,
        patterns::*,
        shape_order::*,
        shape_counter::*,
        traits::*,
    };
}

pub mod pc_possible;

mod bit_shapes;
mod clipped_board;
mod shape_sequence;
mod patterns;
mod shape_order;
mod shape_counter;
mod traits;

mod internal_macros;
mod internals;
