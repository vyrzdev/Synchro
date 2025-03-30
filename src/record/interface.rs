use std::cmp::max;
use nexosim::model::{Context, InitializedModel, Model};
use nexosim::ports::Output;
use nexosim::time::MonotonicTime;
use crate::messages;
use crate::messages::{PollRequest, RecordReply, RecordRequest};
use crate::observations::{DefinitionPredicate, Observation, PlatformMetadata};
use crate::polling::WaitingPoll;
use chrono::{Duration, TimeDelta};

pub struct DeviationSession {
    send: MonotonicTime
}

pub struct RecordInterface {
    max_deviation: Option<TimeDelta>,
    min_deviation: Option<TimeDelta>,
    pub(crate) observation_output: Output<messages::Observation>,
    pub(crate) request_output: Output<messages::RecordRequest>,
    deviation_session: Option<DeviationSession>,
    backoff: std::time::Duration,
    name: String
}

impl RecordInterface {
    pub fn new(name: String, backoff: std::time::Duration) -> RecordInterface {
        RecordInterface {
            max_deviation: None,
            min_deviation: None,
            deviation_session: None,
            observation_output: Default::default(),
            request_output: Default::default(),
            name,
            backoff
        }
    }

    pub async fn reply_input(&mut self, reply: RecordReply, ctx: &mut Context<Self>) {
        match reply {
            RecordReply::Query(events) => {
                for (definition, timestamp, logical_timestamp) in events {
                    let min_time = MonotonicTime::from_chrono_date_time(&(timestamp.to_chrono_date_time(0).unwrap() - self.min_deviation.unwrap()), 0);
                    let max_time = MonotonicTime::from_chrono_date_time(&(timestamp.to_chrono_date_time(0).unwrap() - self.max_deviation.unwrap()), 0);

                    self.observation_output.send(messages::Observation(Observation {
                        interval: (min_time, max_time),
                        definition_predicate: definition,
                        source: self.name.clone(),
                        platform_metadata: PlatformMetadata::Record {
                            logical_version: logical_timestamp
                        },
                    })).await;
                    println!("Generating Events!");
                }

                ctx.schedule_event(self.backoff, Self::query, ()).unwrap();
            }
            RecordReply::Deviation(deviated_clock) => {
                // TODO: Deviation Logic.
                let send_at = self.deviation_session.as_ref().unwrap().send.to_chrono_date_time(0).unwrap();
                let reply_at = ctx.time().to_chrono_date_time(0).unwrap();
                let platform_timestamp = deviated_clock.to_chrono_date_time(0).unwrap();
                let observed_max_deviation = platform_timestamp - send_at;
                let observed_min_deviation = platform_timestamp - reply_at;

                self.max_deviation = Some(observed_max_deviation);
                self.min_deviation = Some(observed_min_deviation);

                ctx.schedule_event(self.backoff, Self::deviation_query, 0).unwrap();
            }
        }
    }

    pub async fn query(&mut self) {
        self.request_output.send(RecordRequest::Query {}).await;
    }

    fn deviation_query<'a>(
        &'a mut self,
        arg: u64,
        cx: &'a mut Context<Self>
    ) -> impl Future<Output=()> + Send + 'a {
        async move {
            /* implementation */
            self.deviation_session = Some(DeviationSession {
                send: cx.time()
            });
            self.request_output.send(RecordRequest::Deviation {}).await;
        }
    }
}


impl Model for RecordInterface {
    async fn init(mut self, ctx: &mut Context<Self>) -> InitializedModel<Self> {
        self.query().await; // Start Events queries
        ctx.schedule_event(ctx.time() + std::time::Duration::from_millis(1000), Self::deviation_query, (0)).unwrap();
        self.into()
    }
}