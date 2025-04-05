use std::time::Duration;
use chrono::TimeDelta;
use nexosim::model::{Context, InitializedModel, Model};
use nexosim::ports::Output;
use serde::{Deserialize, Serialize};
use tai_time::MonotonicTime;
use crate::intervals::Interval;
use crate::observations::Observation;
use crate::ordering::PlatformMetadata;
use crate::simulation::data::SimulationMetaData;
use crate::simulation::messages::{InterfaceQuery, PlatformQuery};
use crate::simulation::record::messages::{RecordQuery, RecordReply};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordInterfaceParameters {
    pub(crate) backoff: Duration,
}

pub struct DeviationState {
    max_deviation: TimeDelta,
    min_deviation: TimeDelta,
    last_sent: Option<MonotonicTime>
}


pub struct RecordInterface {
    name: String,
    config: RecordInterfaceParameters,
    deviation_state: DeviationState,
    pub(crate) observation_output: Output<Observation<MonotonicTime>>,
    pub(crate) query_output: Output<PlatformQuery>,
}


impl RecordInterface {
    pub fn new(name: String, config: RecordInterfaceParameters) -> RecordInterface {
        RecordInterface {
            name,
            config,
            deviation_state: DeviationState {
                max_deviation: TimeDelta::MIN,
                min_deviation: TimeDelta::MAX,
                last_sent: None,
            },
            observation_output: Default::default(),
            query_output: Default::default(),
        }
    }

    pub async fn input(&mut self, reply: RecordReply, ctx: &mut Context<Self>) {
        match reply {
            RecordReply::Query(events) => {
                // Register Observations
                for (definition, deviated_timestamp, logical_version) in events {
                    self.observation_output.send(Observation {
                        interval: self.bound_deviated_timestamp(deviated_timestamp),
                        definition_predicate: definition,
                        source: self.name.clone(),
                        platform_metadata: PlatformMetadata::Simulation(SimulationMetaData {
                            monotonic: logical_version,
                        }),
                    }).await;
                }
                // Schedule Next Query
                ctx.schedule_event(ctx.time() + self.config.backoff, Self::query_events, ()).unwrap()
            },
            RecordReply::Deviation(deviated_clock) => {
                let last_sent = self.deviation_state.last_sent.take().unwrap();

                // Find and update min/max deviation.
                (self.deviation_state.min_deviation, self.deviation_state.max_deviation) = Self::find_deviation(last_sent, ctx.time(), deviated_clock);

                // Schedule next deviation query!
                ctx.schedule_event(ctx.time() + self.config.backoff, Self::query_deviation, ()).unwrap();
            }
        }
    }

    pub fn find_deviation(sent: MonotonicTime, replied: MonotonicTime, deviated: MonotonicTime) -> (TimeDelta, TimeDelta) {
        let max_deviation = deviated.to_chrono_date_time(0).unwrap() - sent.to_chrono_date_time(0).unwrap();
        let min_deviation = deviated.to_chrono_date_time(0).unwrap() - replied.to_chrono_date_time(0).unwrap();

        (min_deviation, max_deviation)
    }

    pub fn bound_deviated_timestamp(&self, deviated_time: MonotonicTime) -> Interval<MonotonicTime> {
        // TODO: VERIFY!
        // p = t + sigma, therefore t = p - sigma
        let min_time = deviated_time.to_chrono_date_time(0).unwrap() - self.deviation_state.max_deviation;
        let max_time = deviated_time.to_chrono_date_time(0).unwrap() - self.deviation_state.min_deviation;

        Interval(MonotonicTime::from_chrono_date_time(&min_time, 0), MonotonicTime::from_chrono_date_time(&max_time, 0))
    }

    pub async fn query_deviation(&mut self, _: (), ctx: &mut Context<Self>) {
        self.deviation_state.last_sent = Some(ctx.time());
        self.query_output.send(PlatformQuery::Interface(InterfaceQuery::Record(RecordQuery::Deviation))).await;
    }

    pub async fn query_events(&mut self, _: ()) {
        // Send platform query!
        self.query_output.send(PlatformQuery::Interface(InterfaceQuery::Record(RecordQuery::Query))).await;
    }
}

impl Model for RecordInterface {
    async fn init(mut self, ctx: &mut Context<Self>) -> InitializedModel<Self> {
        self.query_events(()).await;
        self.query_deviation((), ctx).await;
        self.into()
    }
}



//
// pub struct RecordInterface {
//     max_deviation: Option<TimeDelta>,
//     min_deviation: Option<TimeDelta>,
//     pub(crate) observation_output: Output<messages::Observation>,
//     pub(crate) request_output: Output<messages::RecordRequest>,
//     deviation_session: Option<DeviationSession>,
//     backoff: std::time::Duration,
//     name: String
// }
//
// impl RecordInterface {
//     pub fn new(name: String, backoff: std::time::Duration) -> RecordInterface {
//         RecordInterface {
//             max_deviation: None,
//             min_deviation: None,
//             deviation_session: None,
//             observation_output: Default::default(),
//             request_output: Default::default(),
//             name,
//             backoff
//         }
//     }
//
//     pub async fn reply_input(&mut self, reply: RecordReply, ctx: &mut Context<Self>) {
//         match reply {
//             RecordReply::Query(events) => {
//                 for (definition, timestamp, logical_timestamp) in events {
//                     let min_time = MonotonicTime::from_chrono_date_time(&(timestamp.to_chrono_date_time(0).unwrap() - self.min_deviation.unwrap()), 0);
//                     let max_time = MonotonicTime::from_chrono_date_time(&(timestamp.to_chrono_date_time(0).unwrap() - self.max_deviation.unwrap()), 0);
//
//                     self.observation_output.send(messages::Observation(Observation {
//                         interval: (min_time, max_time),
//                         definition_predicate: definition,
//                         source: self.name.clone(),
//                         platform_metadata: PlatformMetadata::Record {
//                             logical_version: logical_timestamp
//                         },
//                     })).await;
//                     // println!("Generating Events!");
//                 }
//
//                 ctx.schedule_event(self.backoff, Self::query, ()).unwrap();
//             }
//             RecordReply::Deviation(deviated_clock) => {
//                 // TODO: Deviation Logic.
//                 let send_at = self.deviation_session.as_ref().unwrap().send.to_chrono_date_time(0).unwrap();
//                 let reply_at = ctx.time().to_chrono_date_time(0).unwrap();
//                 let platform_timestamp = deviated_clock.to_chrono_date_time(0).unwrap();
//                 let observed_max_deviation = platform_timestamp - send_at;
//                 let observed_min_deviation = platform_timestamp - reply_at;
//
//                 self.max_deviation = Some(observed_max_deviation);
//                 self.min_deviation = Some(observed_min_deviation);
//
//                 ctx.schedule_event(self.backoff, Self::deviation_query, 0).unwrap();
//             }
//         }
//     }
//
//     pub async fn query(&mut self) {
//         self.request_output.send(RecordRequest::Query {}).await;
//     }
//
//     fn deviation_query<'a>(
//         &'a mut self,
//         arg: u64,
//         cx: &'a mut Context<Self>
//     ) -> impl Future<Output=()> + Send + 'a {
//         async move {
//             /* implementation */
//             self.deviation_session = Some(DeviationSession {
//                 send: cx.time()
//             });
//             self.request_output.send(RecordRequest::Deviation {}).await;
//         }
//     }
// }
//
//
// impl Model for RecordInterface {
//     async fn init(mut self, ctx: &mut Context<Self>) -> InitializedModel<Self> {
//         self.query().await; // Start Events queries
//         ctx.schedule_event(ctx.time() + std::time::Duration::from_millis(1000), Self::deviation_query, (0)).unwrap();
//         self.into()
//     }
// }