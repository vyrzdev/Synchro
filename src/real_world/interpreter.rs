use chrono::{Date, DateTime, Utc};
use log::{error, warn};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{Receiver, Sender};
use crate::interpreter::history::History;
use crate::observations::Observation;
use crate::value::Value;

pub async fn interpreter_worker(mut observations_in: Receiver<Observation<DateTime<Utc>>>, initial_value: Value, value_out: Sender<Value>) -> ! {
    let mut history = History::new();

    // Greedily capture all available observations.
    loop {
        match observations_in.recv().await {
            Some(observation) => {
                history.insert(observation);

                loop {
                    match observations_in.try_recv() {
                        Ok(observation) => history.insert(observation),
                        Err(TryRecvError::Disconnected) => panic!("Interpreter :: Observations Input Closed!"),
                        _ => break
                    }
                }

                // Find new value
                match history.apply(Some(initial_value), Utc::now()) {
                    Ok(value) => value_out.send(value).await.unwrap(), // Send to interfaces!
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