use log::debug;
use nexosim::model::{Context, Model};
use nexosim::ports::{EventSlot, Output};
use tai_time::MonotonicTime;
use crate::interpreter::error::ConflictError;
use crate::interpreter::history::History;
use crate::observations::Observation;
use crate::value::Value;

pub struct InterpreterConfig {
    pub(crate) initial_value: Value
}

pub struct Interpreter {
    history: History<MonotonicTime>,
    config: InterpreterConfig,
    pub(crate) found_out: Output<Result<Value, ConflictError<MonotonicTime>>>
}

impl Interpreter {
    pub fn new(config: InterpreterConfig) -> Self {
        Interpreter {
            history: History::new(),
            found_out: Default::default(),
            config
        }
    }

    // TODO: Requestor/buffered port here to improve simulation speed.
    pub (crate) async fn input(&mut self, observation: Observation<MonotonicTime>, ctx: &mut Context<Self>) {
        debug!("Observed {:?} at {}", observation, ctx.time());


        self.history.insert(observation);

        // Send the result of history applied to initial value.
        self.found_out.send(self.history.apply(Some(self.config.initial_value), ctx.time())).await;
    }
}


impl Model for Interpreter {}