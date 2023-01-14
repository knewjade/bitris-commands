use bitris::prelude::*;

use crate::ClippedBoard;

#[derive(Clone, Debug)]
pub(crate) struct VerticalParity {
    parity: [i32; 2],
}

impl VerticalParity {
    pub(crate) fn new(clipped_board: ClippedBoard) -> Self {
        let mut parity: [i32; 2] = [0; 2];
        for y in 0..clipped_board.height() {
            let y = y as i32;
            for x in 0..10 {
                if clipped_board.board_ref().is_free_at(xy(x, y)) {
                    parity[(x % 2) as usize] += 1;
                }
            }
        }
        Self { parity }
    }

    pub(crate) fn place(&self, placement: BlPlacement) -> Self {
        let mut clone = self.clone();
        for location in placement.locations() {
            clone.parity[(location.x % 2) as usize] -= 1;
        }
        clone
    }

    fn fixed_status(&self) -> Option<bool> {
        if self.parity[0] < 0 || self.parity[1] < 0 {
            return Some(false);
        }

        if self.parity == [0, 0] {
            return Some(true);
        }

        None
    }

    pub(crate) fn validates(&self, shapes: &[Shape], index: usize, allows_hold: bool) -> bool {
        debug_assert!(0 < shapes.len());
        debug_assert!((self.parity[0] + self.parity[1]) <= (shapes.len() * 4) as i32);

        if let Some(status) = self.fixed_status() {
            return status;
        }

        let vertical_parity: &[(i32, i32)] = match shapes[index] {
            Shape::T => &[(2, 2), (1, 3)],
            Shape::I => &[(2, 2), (0, 4)],
            Shape::L | Shape::J => &[(1, 3)],
            Shape::O | Shape::S | Shape::Z => &[(2, 2)],
        };

        for (left, right) in vertical_parity {
            {
                let next = Self { parity: [self.parity[0] - left, self.parity[1] - right] };
                if next.validates(shapes, index + 1, allows_hold) {
                    return true;
                }
            }

            if left != right {
                let next = Self { parity: [self.parity[0] - right, self.parity[1] - left] };
                if next.validates(shapes, index + 1, allows_hold) {
                    return true;
                }
            }

            if allows_hold {
                if self.validates(shapes, index + 1, false) {
                    return true;
                }
            }
        }

        false
    }
}
