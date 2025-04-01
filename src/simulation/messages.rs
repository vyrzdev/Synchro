use nexosim::time::MonotonicTime;
use crate::observations;
use crate::observations::DefinitionPredicate;
use crate::value::Value;

#[derive(Clone)]
pub enum PollRequest {
    Query,
    Write(Value),
    SafeWrite(Value, Value)
}

#[derive(Clone)]
pub enum PollReply {
    Query(Value),
    WriteComplete,
    WriteFailure(Value)
}
#[derive(Clone)]
pub struct Observation(pub observations::Observation);
#[derive(Clone)]
pub struct Write {
    pub(crate) value: Value
}
#[derive(Clone)]
pub enum RecordRequest {
    Deviation,
    Query
}
#[derive(Clone)]
pub enum RecordReply {
    Query(Vec<(DefinitionPredicate, MonotonicTime, u64)>),
    Deviation(MonotonicTime)
}


