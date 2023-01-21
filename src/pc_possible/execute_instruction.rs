/// A collection of statements that instruct execution to continue/stop.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub enum ExecuteInstruction {
    #[default] Continue,
    Stop,
}
