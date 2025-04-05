use std::error::Error;
use nexosim::simulation::ExecutionError;
use tai_time::MonotonicTime;
use crate::interpreter::error::ConflictError;
use crate::interpreter::history::History;
use crate::simulation::driver::TruthRecord;

#[derive(Debug)]
pub enum SimulationError {
    Divergence(DivergenceError),
    Conflict(ConflictError<MonotonicTime>),
    Other(Box<dyn Error>),
}

impl From<ExecutionError> for SimulationError {
    fn from(value: ExecutionError) -> Self {
        Self::Other(Box::new(value))
    }
}
#[derive(Debug)]
pub struct DivergenceError {
    pub(crate) diverged_at: MonotonicTime,
    pub(crate) truth: Vec<TruthRecord>, // Whole true sequence of events.
    pub(crate) history: History<MonotonicTime> // Whole history for divergence.
}