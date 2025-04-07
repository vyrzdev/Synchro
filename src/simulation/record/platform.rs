use nexosim::model::{Context, Model};
use nexosim::ports::Output;
use nexosim::time::MonotonicTime;
use chrono::TimeDelta;
use serde::{Deserialize, Serialize};
use crate::core::predicates::DefinitionPredicate;
use crate::simulation::driver::TruthRecord;
use crate::simulation::messages::{InterfaceQuery, PlatformQuery, UserAction};
use crate::simulation::record::messages::{RecordQuery, RecordReply};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordPlatformParameters {
    pub(crate) deviation: TimeDelta,
}


// Simple event record- outputs definition, time, and logical version.
pub type Event = (DefinitionPredicate, MonotonicTime, u64);
pub struct RecordPlatform {
    name: String,
    logical_version: u64,
    events_since: Vec<Event>,
    config: RecordPlatformParameters,
    pub(crate) reply_output: Output<RecordReply>,
    pub(crate) truth_output: Output<TruthRecord>,
}

impl RecordPlatform {
    pub fn new(name: String, config: RecordPlatformParameters) -> RecordPlatform {
        RecordPlatform {
            name,
            logical_version: 0,
            events_since: vec![],
            config,
            reply_output: Output::default(),
            truth_output: Output::default(),
        }
    }

    pub async fn input(&mut self, query: PlatformQuery, ctx: &mut Context<Self>) {
        match query {
            PlatformQuery::User(user_action) => match user_action {
                UserAction::Mutation(delta) => {
                    self.events_since.push(
                        (DefinitionPredicate::AllMut(delta), self.deviate_time(ctx.time()), self.logical_version)
                    );
                    self.truth_output.send((DefinitionPredicate::AllMut(delta), ctx.time())).await;
                    self.logical_version += 1;
                },
                UserAction::Assignment(new) => {
                    self.events_since.push(
                        (DefinitionPredicate::LastAssn(new), self.deviate_time(ctx.time()), self.logical_version)
                    );
                    self.truth_output.send((DefinitionPredicate::LastAssn(new), ctx.time())).await;
                    self.logical_version += 1;
                }
            },
            PlatformQuery::Interface(InterfaceQuery::Record(record_query)) => match record_query {
                RecordQuery::Query => {
                    self.reply_output.send(RecordReply::Query(self.events_since.clone())).await;
                    self.events_since = Vec::with_capacity(self.events_since.len());
                },
                RecordQuery::Deviation => {
                    // Send reply with deviation applied.
                    self.reply_output.send(RecordReply::Deviation(self.deviate_time(ctx.time()))).await;
                }
            },
            x => panic!("Unexpected query type! {x:#?}")
        }
    }

    pub fn deviate_time(&self, time: MonotonicTime) -> MonotonicTime {
        // Note- Hacky, we have to use Chrono for negative deviations.
        MonotonicTime::from_chrono_date_time(&(
            time.to_chrono_date_time(0).unwrap() + self.config.deviation
        ), 0)
    }

}

impl Model for RecordPlatform {}