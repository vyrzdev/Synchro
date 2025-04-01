use std::time::Duration;
use nexosim::model::{Context, InitializedModel, Model};
use nexosim::ports::Output;
use nexosim::time::MonotonicTime;
use rand::Rng;
use rand_distr::{Exp, Normal};
use crate::simulation::messages::{PollRequest, RecordReply, RecordRequest};
use crate::observations::DefinitionPredicate;
use crate::simulation::polling::safe_platform::SafePollingPlatform;
use crate::simulation::record::interface::RecordInterface;
use crate::TruthRecord;

pub struct RecordPlatform {
    events_since: Vec<(DefinitionPredicate, MonotonicTime, u64)>,
    deviation_distribution: Normal<f64>,
    processing_distribution: Normal<f64>,
    sale_distribution: Exp<f64>,
    pub(crate) output: Output<RecordReply>,
    pub(crate) truth_output: Output<TruthRecord>,
    logical_version: u64
}


impl RecordPlatform {
    pub fn new(deviation: f64, std_dev: f64, avg_proc: f64, hourly_sales: f64) -> Self {
        RecordPlatform {
            events_since: vec![],
            processing_distribution: Normal::new(avg_proc, 1.0).unwrap(), // Processing time is normally distributed.
            deviation_distribution: Normal::new(deviation, std_dev).unwrap(), // We assume deviation is normally distributed.
            sale_distribution: Exp::new(hourly_sales/60.0/60.0/1000.0).unwrap(), // Sales are Exponential Distributed (avg lambda)            output: Default::default(),
            output: Default::default(),
            truth_output: Default::default(),
            logical_version: 0
        }
    }

    pub fn input(&mut self, request: RecordRequest, ctx: &mut Context<Self>) {
        // No need to model writes- as have NO EFFECT on records!
        match request {
            RecordRequest::Query => {
                ctx.schedule_event(self.proc_delay(), Self::query_reply, ()).unwrap();
            },
            RecordRequest::Deviation => {
                let proc_delay = self.proc_delay();
                let returned_clock = self.get_deviated_clock(ctx.time() + proc_delay);
                ctx.schedule_event(self.proc_delay(), Self::deviation_reply, (returned_clock)).unwrap();
            }
        }
    }

    pub async fn query_reply(&mut self) {
        self.output.send(RecordReply::Query(self.events_since.clone())).await;
        self.events_since = Vec::new();
    }

    pub async fn deviation_reply(&mut self, returned_clock: MonotonicTime) {
        self.output.send(RecordReply::Deviation(returned_clock)).await;
    }
    pub fn proc_delay(&self) -> Duration {
        Duration::from_millis(rand::rng().sample(self.processing_distribution).round() as u64)
    }

    pub fn sale_after(&self) -> Duration {
        Duration::from_millis(rand::rng().sample(self.sale_distribution).round() as u64)
    }

    pub fn get_deviated_clock(&mut self, now: MonotonicTime) -> MonotonicTime {
        now + Duration::from_millis(rand::rng().sample(self.deviation_distribution).round() as u64)
    }
    fn make_sale<'a>(
        &'a mut self,
        arg: u64,
        cx: &'a mut Context<Self>
    ) -> impl Future<Output=()> + Send + 'a {
        async move {
            /* implementation */
            let deviated = self.get_deviated_clock(cx.time());
            self.truth_output.send((DefinitionPredicate::AllMut(-1), cx.time())).await;
            self.events_since.push((DefinitionPredicate::AllMut(-1), deviated, self.logical_version));
            cx.schedule_event(self.sale_after(), Self::make_sale, (0)).unwrap();
            self.logical_version += 1;
        }
    }
}

impl Model for RecordPlatform {
    async fn init(self, ctx: &mut Context<Self>) -> InitializedModel<Self> {
        // Give time for deviation estimation.
        ctx.schedule_event(Duration::from_millis(1000) + self.sale_after(), Self::make_sale, (0)).unwrap();
        self.into()
    }
}