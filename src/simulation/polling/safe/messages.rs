use crate::simulation::messages::UserAction;
use crate::value::Value;

#[derive(Debug, Copy, Clone)]
pub enum SafePollReply {
    Query(Value), // Query response- value at time of processing.
    WriteSuccess, // Write succeeded- no change since supplied value.
    WriteFail(Value) // Write failed as has changed since supplied value. Return new value.
}

#[derive(Debug, Copy, Clone)]
pub enum SafePollQuery {
    Query, // Query the Platform State
    Write(Value, Value) // Write attempt 0: to write, 1: if is still this.
}