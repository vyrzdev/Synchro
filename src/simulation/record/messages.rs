use tai_time::MonotonicTime;
use crate::simulation::record::platform::Event;
use crate::value::Value;

#[derive(Debug, Clone)]
pub enum RecordReply {
    Query(Vec<Event>), // Query response- events since last query.
    Deviation(MonotonicTime) // Query response - clock value at time of receipt.
}

#[derive(Debug, Copy, Clone)]
pub enum RecordQuery {
    Query, // Query the Platform State
    Deviation // Query the platform's current clock value.
}