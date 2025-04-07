use std::collections::LinkedList;
use nexosim::model::{Context, Model};
use nexosim::ports::{Output};
use tai_time::MonotonicTime;
use crate::core::interpreter::error::ConflictError;
use crate::core::interpreter::history::History;
use crate::core::observations::Observation;
use crate::value::Value;

pub struct InterpreterConfig {
    pub(crate) initial_value: Value
}


/// Interpreter model for DES SIMULATION.
pub struct Interpreter {
    history: History<MonotonicTime>,
    config: InterpreterConfig,
    stable_value: Option<Value>,
    pub(crate) found_out: Output<Result<Value, ConflictError<MonotonicTime>>>
}

impl Interpreter {
    pub fn new(config: InterpreterConfig) -> Self {
        Interpreter {
            history: History::new(),
            stable_value: Some(config.initial_value),
            found_out: Default::default(),
            config
        }
    }

    /// Input function for interpreter model.
    /// TODO: Requestor/buffered port here to improve simulation speed.
    pub (crate) async fn input(&mut self, observation: Observation<MonotonicTime>, ctx: &mut Context<Self>) {
        // debug!("Observed {:?} at {}", observation, ctx.time());

        // info!("Had: {:?}", self.stable_value);
        // info!("Got: {observation:?}");

        let mut pruned = History {
            list: LinkedList::from_iter(self.history.insert(observation, ctx.time()).into_iter()),
        };


        // info!("Pruning: {pruned:?}");
        self.stable_value = match pruned.apply(self.stable_value, ctx.time()) {
            Ok(value) => Some(value),
            Err(_) => {
                None
            }
        };

        // info!("Now Have: {:?}", self.stable_value);
        let result = self.history.apply(self.stable_value, ctx.time());
        // info!("Sending: {:?}", result);
        // Send the result of history applied to initial value.
        self.found_out.send(result).await;
    }
}


impl Model for Interpreter {}