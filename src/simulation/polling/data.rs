use tai_time::MonotonicTime;
use crate::value::Value;
// Common datastructures between safe and unsafe polling.

pub struct SentPoll {
    pub(crate) at: MonotonicTime
}
pub struct FinishedPoll {
    pub(crate) sent: MonotonicTime,
    pub(crate) value: Value,
}

pub struct PollState {
    pub(crate) ordering: u64, // Monotonic logical ordering.
    pub(crate) current: Option<SentPoll>,
    pub(crate) last: FinishedPoll
}

pub struct WriteState {
    pub(crate) sent: MonotonicTime,
    pub(crate) value: Value,
}
