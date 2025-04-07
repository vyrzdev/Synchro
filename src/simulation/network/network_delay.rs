use std::time::Duration;
use nexosim::model::{Context, Model};
use nexosim::ports::{Output};
use rand::Rng;
use rand_distr::Pareto;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkParameters {
    /// Avg milliseconds one-way RTT.
    pub(crate) size: f64,
    /// Scale of Pareto - Larger = less extremes.
    pub(crate) scale: f64,
}

/// Network Delay Model - Uses the Pareto Distribution
pub struct NetworkConnection<MessageType1: Clone + Send + Sync + 'static, MessageType2:  Clone + Send + Sync + 'static> {
    pub output_1: Output<MessageType1>,
    pub output_2: Output<MessageType2>,
    distribution: Pareto<f64>,
}
impl<MessageType1: Clone + Send + Sync + 'static, MessageType2: Clone + Send + Sync + 'static> NetworkConnection<MessageType1, MessageType2> {
    pub fn new(network_parameters: NetworkParameters) -> Self {
        NetworkConnection {
            output_1: Output::default(),
            output_2: Output::default(),
            // Pareto Distribution- as per: http://blog.simiacryptus.com/posts/modeling_network_latency/
            distribution: Pareto::new(network_parameters.size, network_parameters.scale).unwrap()
        }
    }

    /// When get input- schedule output for after network delay.
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

    /// When get input- schedule output for after network delay.
    pub fn input_2(&mut self, value: MessageType2, ctx: &mut Context<Self>) {
        ctx.schedule_event(self.delay(), Self::send_2, value).unwrap();
    }

    pub fn delay(&mut self) -> Duration {
        let net_delay = Duration::from_millis(rand::rng().sample(self.distribution).round() as u64);
        // println!("Delay: {net_delay:?}");
        net_delay
    }
}
impl<MessageType1: Clone + Sync + Send, MessageType2: Clone + Sync + Send> Model for NetworkConnection<MessageType1, MessageType2> {}