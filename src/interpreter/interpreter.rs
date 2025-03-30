use std::time::Duration;
use nexosim::model::{Context, Model};
use nexosim::simulation::ActionKey;
use crate::messages;
use crate::processing::{traverse_history, History, Node};

pub struct Interpreter {
    history: History,
    processing_delay: u64,
    observation_count: u64,
    pending: bool
}

impl Interpreter {
    pub fn new(processing_delay: u64) -> Self {
        Interpreter {
            history: History::new(),
            processing_delay,
            observation_count: 0,
            pending: false,
        }
    }

    pub(crate) async fn input(&mut self, observation: messages::Observation, ctx: &mut Context<Self>) {
        self.history.add_node(Node::new(observation.observation, self.observation_count));
        self.observation_count+=1; // Monotonic Reference Name
        if !self.pending {
            ctx.schedule_event(Duration::from_millis(self.processing_delay), Self::process, ()).unwrap();
            self.pending = true;
        }
    }

    async fn process(&mut self) {
        traverse_history(&mut self.history);
        self.pending = false;
    }
}


impl Model for Interpreter {}