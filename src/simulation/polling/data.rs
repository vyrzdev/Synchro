use tai_time::MonotonicTime;
use crate::value::Value;
// Common datastructures between safe and unsafe polling.

#[derive(Debug)]
pub struct SentPoll {
    pub(crate) at: MonotonicTime
}

#[derive(Debug)]
pub struct FinishedPoll {
    pub(crate) sent: MonotonicTime,
    pub(crate) value: Value,
}

#[derive(Debug)]
pub struct PollState {
    pub(crate) ordering: u64, // Monotonic logical ordering.
    pub(crate) current: Option<SentPoll>,
    pub(crate) last: FinishedPoll
}

#[derive(Debug)]
pub struct WriteState {
    pub(crate) sent: MonotonicTime,
    pub(crate) value: Value,
}
