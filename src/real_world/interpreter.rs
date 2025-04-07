use std::collections::LinkedList;
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver};
use tokio::sync::watch::Sender;
use crate::core::interpreter::history::History;
use crate::core::observations::Observation;
use crate::value::Value;


/// This function processes a stream of `Observation` inputs,
/// maintains an observation history, and calculates a stable output `Value`
/// based on this history.
///
/// # Parameters
/// - `observations_in`: An receiver channel to fetch incoming `Observation` values.
/// - `initial_value`: The initial `Value` to start things off.
/// - `value_out`: A sender channel to send the computed `Value` back to interfaces.
///
/// # Errors
/// - Panics if the `observations_in` channel is closed or disconnected.
/// - Logs errors and warnings in the event of conflicts during applying history.
///
/// # Notes
/// - This function runs indefinitely.
pub async fn interpreter_worker(mut observations_in: Receiver<Observation<DateTime<Utc>>>, initial_value: Value, value_out: Sender<Option<Value>>) -> ! {
    let mut history = History::new(); // Init History

    // Capture observations as they become available.
    let mut stable_value = Some(initial_value);
    loop {
        match observations_in.recv().await {
            Some(observation) => {
                info!("Interpreter - Got Observation: {observation:?}");

                // Take and apply all pruned observations (prune occurs on write)
                let mut pruned = History {
                    list: LinkedList::from_iter(history.insert(observation, Utc::now()).into_iter()),
                };
                // New basis is after pruned.
                stable_value = match pruned.apply(stable_value, Utc::now()) {
                    Ok(value) => Some(value),
                    Err(_) => {
                        None
                    }
                };

                loop {
                    // Greedily capture all available observations. (efficiency)
                    match observations_in.try_recv() {
                        Ok(observation) => {
                            let mut pruned = History {
                                list: LinkedList::from_iter(history.insert(observation, Utc::now()).into_iter()),
                            };
                            stable_value = match pruned.apply(stable_value, Utc::now()) {
                                Ok(value) => Some(value),
                                Err(_) => {
                                    None
                                }
                            };
                        },
                        Err(TryRecvError::Disconnected) => panic!("Interpreter :: Observations Input Closed!"),
                        _ => break
                    }
                }

                // Find new value
                match history.apply(stable_value, Utc::now()) {
                    Ok(value) => {
                        info!("Calculated Result: {}, sending!", value);
                        value_out.send(Some(value)).unwrap();
                    }, // Send to interfaces!
                    Err(conflict) => {
                        error!("Interpreter :: Conflict Error {:#?}", conflict);
                        warn!("Interpreter :: Will not synchronize until resolved!");
                    }
                }
            },
            None => panic!("Interpreter :: Observations Input Closed!")
        }
    }
}