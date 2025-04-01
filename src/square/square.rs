#[derive(PartialEq, Copy, Clone, Debug)]
pub struct SquareMetadata {
    logical_version: u64
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum PollingInterpretation {
    Transition,
    Mutation,
    Assignment
}