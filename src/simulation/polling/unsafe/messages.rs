use crate::value::Value;

#[derive(Debug, Copy, Clone)]
pub enum UnsafePollReply {
    Query(Value), // Query response- value at time of processing.
    WriteComplete, // Write succeeded- no change since supplied value.
}

#[derive(Debug, Copy, Clone)]
pub enum UnsafePollQuery {
    Query, // Query the Platform State
    Write(Value) // Value to write.
}