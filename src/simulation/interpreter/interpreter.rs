use std::time::Duration;
use nexosim::model::{Context, Model};
use nexosim::ports::{EventSlot, Output};
use crate::value::Value;

pub struct Interpreter {
    history: History,
    processing_delay: u64,
    observation_count: u64,
    pending: bool,
    pub(crate) found_out: Output<Option<Value>>
}

impl Interpreter {
    pub fn new(processing_delay: u64) -> Self {
        Interpreter {
            history: History::new(),
            processing_delay,
            observation_count: 0,
            pending: false,
            found_out: Output::new(),
        }
    }

    // pub(crate) async fn input(&mut self, observation: messages::Observation, ctx: &mut Context<Self>) {
    //     self.history.add_node(Node::new(observation.0, self.observation_count));
    //     self.observation_count+=1; // Monotonic Reference Name
    //     if !self.pending {
    //         ctx.schedule_event(Duration::from_millis(self.processing_delay), Self::process, ()).unwrap();
    //         self.pending = true;
    //     }
    // }

    // async fn process(&mut self) {
    //     self.found_out.send( traverse_history(&mut self.history, Some(10))).await;
    //     // TODO:
    //     // if conflict
    //     // self.conflict_out write. - Stops simulation. Marks conflict time. Over.
    //     self.pending = false;
    // }
}


impl Model for Interpreter {}