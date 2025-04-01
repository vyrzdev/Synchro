use nexosim::time::MonotonicTime;
use crate::value::Value;

pub mod unsafe_interface;
pub mod unsafe_platform;
pub mod safe_platform;
pub mod safe_interface;

pub enum PollingInterpretation {
    Transition,
    AllMut,
    LastAssn
}

#[derive(Clone)]
pub struct WaitingPoll {
    send: MonotonicTime
}
#[derive(Clone)]
pub struct CompletedPoll {
    pub(crate) send: MonotonicTime,
    pub(crate) receive: MonotonicTime,
    pub(crate) value: Value
}
#[derive(Clone)]
pub struct WaitingWrite {
    send: MonotonicTime,
    value: Value
}