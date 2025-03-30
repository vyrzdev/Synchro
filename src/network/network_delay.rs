use std::time::Duration;
use nexosim::model::{Context, Model};
use nexosim::ports::{EventSlot, Output};
use nexosim::simulation::{Mailbox, SimInit, SimulationError};
use nexosim::time::MonotonicTime;
use rand::distr::Distribution;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand_distr::Pareto;

pub struct NetworkConnection<MessageType1: Clone + Send + Sync + 'static, MessageType2:  Clone + Send + Sync + 'static> {
    pub output_1: Output<MessageType1>,
    pub output_2: Output<MessageType2>,
    distribution: Pareto<f64>,
}
impl<MessageType1: Clone + Send + Sync + 'static, MessageType2: Clone + Send + Sync + 'static> NetworkConnection<MessageType1, MessageType2> {
    pub fn new(avg_rtt: f64, shape: f64) -> Self {
        NetworkConnection {
            output_1: Output::default(),
            output_2: Output::default(),
            distribution: Pareto::new(avg_rtt, shape).unwrap(), // Pareto Distribution- as per: http://blog.simiacryptus.com/posts/modeling_network_latency/
        }
    }

    pub fn input_1(&mut self, value: MessageType1, ctx: &mut Context<Self>) {
        // When get input- schedule output for after network delay.
        ctx.schedule_event(self.delay(), Self::send_1, value).unwrap();
    }

    pub async fn send_1(&mut self, value: MessageType1) {
        self.output_1.send(value).await;
    }

    pub async fn send_2(&mut self, value: MessageType2) {
        self.output_2.send(value).await;
    }

    pub fn input_2(&mut self, value: MessageType2, ctx: &mut Context<Self>) {
        // When get input- schedule output for after network delay.
        ctx.schedule_event(self.delay(), Self::send_2, value).unwrap();
    }

    pub fn delay(&mut self) -> Duration {
        let net_delay = Duration::from_millis(rand::rng().sample(self.distribution).round() as u64);
        if net_delay.as_millis() > 300 {
            println!("LongAssDelay: {}", net_delay.as_millis());
        }
        net_delay
    }
}
impl<MessageType1: Clone + Sync + Send, MessageType2: Clone + Sync + Send> Model for NetworkConnection<MessageType1, MessageType2> {}