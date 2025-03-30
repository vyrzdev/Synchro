use crate::observations;
use crate::value::Value;

#[derive(Clone)]
pub enum PollRequest {
    Query,
    Write(Value)
}

#[derive(Clone)]
pub enum PollReply {
    Query(Value),
    WriteComplete,
}
#[derive(Clone)]
pub struct Observation {
    pub(crate) observation: observations::Observation
}

#[derive(Clone)]
pub struct Write {
    pub(crate) value: Value
}