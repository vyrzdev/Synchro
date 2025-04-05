use std::collections::LinkedList;
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver};
use tokio::sync::watch::Sender;
use crate::interpreter::history::History;
use crate::observations::Observation;
use crate::value::Value;

pub async fn interpreter_worker(mut observations_in: Receiver<Observation<DateTime<Utc>>>, initial_value: Value, value_out: Sender<Option<Value>>) -> ! {
    let mut history = History::new();

    let mut stable_value = Some(initial_value);
    // Greedily capture all available observations.
    loop {
        match observations_in.recv().await {
            Some(observation) => {
                info!("Interpreter - Got Observation: {observation:?}");
                history.insert(observation, Utc::now());

                loop {
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