use std::time::Duration;
use tai_time::MonotonicTime;
use crate::core::interpreter::error::ConflictError;
use crate::simulation::error::DivergenceError;

#[derive(Debug)]
pub struct SimulationResults {
    pub(crate) iterations: u64, // Number of iterations for this simulation.
    pub(crate) conflicts: Vec<ConflictError<MonotonicTime>>, // We log conflict causes for later inspection.
    pub(crate) divergence: Vec<DivergenceError>, // We log whole history for divergence.
    pub(crate) statistics: SimulationStatistics, // Overall Statistics for Simulation
}

#[derive(Debug)]
pub struct SimulationStatistics {
    pub(crate) conflict_number: u64, // Number of conflicts. (USEFULNESS)
    pub(crate) success_rate: u64, // Number of terminations
    pub(crate) divergence_number: u64, // Number of divergences (INCORRECTNESS)
    pub(crate) average_time_to_conflict: Option<Duration>, // Average time taken for a conflict to arise when it does.
    pub(crate) average_time_to_divergence: Option<Duration>, // Average time taken for divergence to happen, when it does.
}
