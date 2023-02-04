pub use bit_shapes::*;
pub use clipped_board::*;
pub use hold_expanded_pattern::*;
pub use hold_expanded_pattern_shape_matcher::*;
pub use pattern::*;
pub use pattern_shape_matcher::*;
pub use shape_counter::*;
pub use shape_order::*;
pub use shape_sequence::*;
pub use traits::*;

#[doc(hidden)]
pub mod prelude {
    pub use bitris::prelude::*;

    pub use crate::{
        bit_shapes::*,
        clipped_board::*,
        pattern::*,
        shape_counter::*,
        shape_order::*,
        shape_sequence::*,
        traits::*,
    };
}

pub mod pc_possible;
pub mod all_pcs;

mod bit_shapes;
mod clipped_board;
mod hold_expanded_pattern_shape_matcher;
mod hold_expanded_pattern;
mod pattern;
mod pattern_shape_matcher;
mod shape_counter;
mod shape_order;
mod shape_sequence;
mod traits;

mod internal_macros;
mod internals;
