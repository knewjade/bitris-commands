// use std::rc::Rc;
//
// use bitris::prelude::*;
// use bitris::srs::SrsKickTable;
//
// use crate::{ClippedBoard, ShapeOrder};
// use crate::pc_possible::PcPossibleExecutorCreationError;
//
// /// The binder to hold and tie settings for `PcPossibleExecutor`.
// #[derive(Clone, PartialEq, PartialOrd, Hash, Debug)]
// pub struct PcPossibleExecutorBinder<T: RotationSystem> {
//     pub move_rules: MoveRules<T>,
//     pub clipped_board: ClippedBoard,
//     pub shape_order: Rc<ShapeOrder>,
//     pub allows_hold: bool,
// }
//
// impl PcPossibleExecutorBinder<SrsKickTable> {
//     /// Making the executor with SRS. See `PcPossibleExecutorBinder::default()` for more details.
//     pub fn srs(move_type: MoveType) -> Self {
//         PcPossibleExecutorBinder::default(MoveRules::srs(move_type))
//     }
// }
//
// impl<T: RotationSystem> PcPossibleExecutorBinder<T> {
//     /// Making the executor with default.
//     ///
//     /// The default values are as follows:
//     ///   + [required] move rules: from argument
//     ///   + [required] shape_order: empty order. You must set this.
//     ///   + board: blank
//     ///   + height: 4 lines
//     ///   + allows hold: yes
//     pub fn default(move_rules: MoveRules<T>) -> Self {
//         Self {
//             move_rules,
//             clipped_board: ClippedBoard::try_new(Board64::blank(), 4).unwrap(),
//             shape_order: Rc::from(ShapeOrder::new(vec![])),
//             allows_hold: true,
//         }
//     }
// }
//
// impl<'a, T: RotationSystem> TryBind<'a, PcPossibleExecutor<'a, T>> for PcPossibleExecutorBinder<T> {
//     type Error = PcPossibleExecutorCreationError;
//
//     fn try_bind(&'a self) -> Result<PcPossibleExecutor<'a, T>, Self::Error> {
//         PcPossibleExecutor::try_new(
//             &self.move_rules,
//             self.clipped_board,
//             self.shape_order.as_ref(),
//             self.allows_hold,
//         )
//     }
// }
//
//
// #[cfg(test)]
// mod tests {
//     use std::rc::Rc;
//     use std::str::FromStr;
//
//     use bitris::prelude::*;
//
//     use crate::{ClippedBoard, ShapeOrder, TryBind};
//     use crate::pc_possible::PcPossibleExecutorBinder;
//
//     #[test]
//     fn reuse() {
//         use Shape::*;
//
//         let mut binder = PcPossibleExecutorBinder::srs(MoveType::Softdrop);
//         let board = Board64::from_str("
//             ..........
//             ....####..
//             ....######
//             ....######
//         ").unwrap();
//         binder.clipped_board = ClippedBoard::try_new(board, 4).unwrap();
//
//         binder.shape_order = Rc::new(ShapeOrder::new(vec![
//             I, O, T, Z, S, J, L,
//         ]));
//         assert!(binder.try_bind().unwrap().execute());
//
//         binder.shape_order = Rc::new(ShapeOrder::new(vec![
//             T, Z, S, J, L, I, O,
//         ]));
//         assert!(!binder.try_bind().unwrap().execute());
//     }
// }
