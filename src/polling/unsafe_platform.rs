use std::time::Duration;
use nexosim::model::{Context, InitializedModel, Model};
use nexosim::ports::Output;
use rand::Rng;
use rand_distr::{Exp, Normal};
use crate::messages::{PollReply, PollRequest};
use crate::value::Value;

pub struct UnsafePollingPlatform {
    current_value: Value,
    processing_distribution: Normal<f64>,
    sale_distribution: Exp<f64>,
    pub(crate) output: Output<PollReply>
}
impl UnsafePollingPlatform {
    pub fn new(initial_value: Value, avg_proc: f64, hourly_sales: f64) -> UnsafePollingPlatform {
        UnsafePollingPlatform {
            current_value: initial_value,
            processing_distribution: Normal::new(avg_proc, 1.0).unwrap(), // Processing time is normally distributed.
            // hourly_sales / 60 / 60 / 1000 == Millisecondly Sales
            sale_distribution: Exp::new(hourly_sales/60.0/60.0/1000.0).unwrap(), // Sales are Exponential Distributed (avg lambda)
            output: Output::default(),
        }
    }

    pub fn input(&mut self, request: PollRequest, ctx: &mut Context<Self>) {
        match request {
            PollRequest::Query => {
                // When get poll request- schedule poll reply.
                ctx.schedule_event(self.proc_delay(), Self::poll_reply, ()).unwrap()
            }
            PollRequest::Write(value) => {
                // When get write request- schedule write and subsequent reply.
                ctx.schedule_event(self.proc_delay(), Self::write, value).unwrap()
            }
        }
    }

    pub async fn write(&mut self, value: Value) {
        // Do write, and then send reply.
        self.current_value = value;
        self.output.send(PollReply::WriteComplete).await;
    }

    pub async fn poll_reply(&mut self) {
        // When processing delay is over - send reply.
        self.output.send(PollReply::Query(self.current_value)).await;
    }
    pub fn proc_delay(&self) -> Duration {
        Duration::from_millis(rand::rng().sample(self.processing_distribution).round() as u64)
    }

    pub fn sale_after(&self) -> Duration {
        Duration::from_millis(rand::rng().sample(self.sale_distribution).round() as u64)
    }

    fn make_sale<'a>(
        &'a mut self,
        arg: u64,
        cx: &'a mut Context<Self>
    ) -> impl Future<Output=()> + Send + 'a {
        async move {
            /* implementation */
            self.current_value -= 1;
            cx.schedule_event(self.sale_after(), Self::make_sale, (0)).unwrap();
        }
    }
}
impl Model for UnsafePollingPlatform {
    async fn init(self, ctx: &mut Context<Self>) -> InitializedModel<Self> {
        ctx.schedule_event(self.sale_after(), Self::make_sale, (0)).unwrap();
        self.into()
    }
}