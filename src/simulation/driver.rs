use std::time::Duration;
use log::{error, info, warn};
use nexosim::ports::{EventBuffer, EventSlot};
use tai_time::MonotonicTime;
use crate::interpreter::history::History;
use crate::predicates::DefinitionPredicate;
use crate::simulation::config::SimulationConfig;
use crate::simulation::error::{DivergenceError, SimulationError};
use crate::simulation::model::build_model;
use crate::simulation::results::{SimulationResults, SimulationStatistics};
pub type TruthRecord = (DefinitionPredicate, MonotonicTime);

fn iteration(simulation_config: &SimulationConfig) -> Result<Option<Duration>, SimulationError> {
    let mut truth_sink = EventBuffer::new(); // Get true event records.
    let mut found_slot = EventSlot::new(); // Where calculated values go for comparison.
    let mut simulation = build_model(simulation_config, &mut truth_sink, &found_slot);

    // Error-Trace Capture
    let mut truth_records = vec![];

    // Statistics Capture
    let mut convergence_times = Vec::new();

    // Main Simulation Loop.
    let mut diverged_at = None; // When the simulation last diverged.
    let mut true_value = simulation_config.initial_value; // Calculated true-value.
    let mut observed_value = None;
    // TODO: Detect Liveness.

    while simulation.time() < (simulation_config.until + Duration::from_secs(60)) {
        simulation.step()?; // Advance simulation.

        // Consume and apply all true events at the moment when they occur.
        for (event, at) in &mut truth_sink {
            truth_records.push((event.clone(), at));
            true_value = event.apply(Some(true_value)).unwrap(); // All true events are known, and defined for all inputs.
            // debug!("True Event {:?} -> {} at {at:?}", event, true_value);
        }

        // Consume and log interpreted values.
        if let Some(observed) = found_slot.next() {
            // debug!("Value Observed: {:?}", observed);
            match observed {
                // If value - update observed.
                Ok(value) => {
                    observed_value = Some(value);
                    match diverged_at {
                        // If has diverged, and has now converged.
                        Some(t) if value == true_value => {
                            // info!("Converged after: {:?}", simulation.time().duration_since(t));
                            // Log time taken to converge.
                            convergence_times.push(simulation.time().duration_since(t)); // t < now
                            diverged_at = None; // Reset divergence counter.\

                        },
                        // If has not been divergent, and has now diverged.
                        None if value != true_value => {
                            // info!("DIVERGED AT {}", simulation.time());
                            // Log that fact. (flips switch)
                            diverged_at = Some(simulation.time());
                        },
                        _ => () // otherwise, pass.
                    }
                },
                // If conflict- Terminate simulation!
                Err(conflict) => return Err(SimulationError::Conflict(conflict)),
            }
        }
    }

    if diverged_at.is_some() {
        // info!("TruthRecord: {truth_records:?}");
        info!("Truth: {true_value:?}");
        info!("Observed: {observed_value:?}");

        return Err(SimulationError::Divergence(DivergenceError {
            diverged_at: diverged_at.unwrap(),
            truth: truth_records,
            history: History::new(), // TODO: Wire up History (not so easy)
        }))
    }

    if convergence_times.is_empty() {
        Ok(None)
    } else {
        Ok(Some(convergence_times.iter().sum::<Duration>() / convergence_times.len() as u32))
    }
}

pub fn driver(
    simulation_config: SimulationConfig, // The simulation to run.
    iterations: u64, // Number of iterations.
) -> SimulationResults {
    let mut divergence = Vec::new();
    let mut conflicts = Vec::new();
    let mut convergence_times = Vec::new();
    let mut success = 0;

    for i in 0..iterations {
        info!("Running Iteration {i}");
        println!("Running Iteration {i}");

        match iteration(&simulation_config) {
            Ok(x) => {
                success += 1;
                info!("Simulation Iteration {i} ended with Success!");
                if let Some(convergence_time) = x {
                    convergence_times.push(convergence_time);
                }
            },
            Err(e) => match e {
                SimulationError::Divergence(error) => {
                    info!("Simulation Iteration {i} ended with Divergence!");
                    info!("");
                    info!("Error Logged to File TODO"); // TODO: Wire up error output.
                    info!("Divergence at: {:?}", error.diverged_at);
                    divergence.push(error);
                }
                SimulationError::Conflict(conflict) => {
                    info!("Simulation Iteration {i} ended with Conflict!");
                    info!("Reason: {conflict:#?}");
                    conflicts.push(conflict);
                }
                SimulationError::Other(x) => {
                    error!("Simulation Iteration {i} failed with error:");
                    error!("{x:#?}");
                    warn!("Sandboxing, and continuing simulation!");
                }
            }
        }
    }

    SimulationResults {
        statistics: SimulationStatistics {
            conflict_number: conflicts.len() as u64,
            success_rate: success,
            divergence_number: divergence.len() as u64,
            average_time_to_conflict: if conflicts.is_empty() {
                None
            } else {
                Some(conflicts.iter().map(|c| c.at.duration_since(MonotonicTime::EPOCH)).sum::<Duration>()/ conflicts.len() as u32)
            },
            average_time_to_divergence: if divergence.is_empty() {
                None
            } else {
                Some(divergence.iter().map(|d| d.diverged_at.duration_since(MonotonicTime::EPOCH)).sum::<Duration>()/ divergence.len() as u32)
            }
        },
        iterations,
        conflicts,
        divergence,
    }
}
