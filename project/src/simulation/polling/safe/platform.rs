use std::time::Duration;
use nexosim::model::{Context, Model};
use nexosim::ports::Output;
use crate::core::predicates::DefinitionPredicate;
use crate::simulation::messages::{InterfaceQuery, PlatformQuery, UserAction};
use crate::simulation::driver::TruthRecord;
use crate::simulation::polling::safe::messages::{SafePollQuery, SafePollReply};
use crate::value::Value;

pub struct SafePollingPlatform {
    name: String,
    current_value: Value,
    pub(crate) reply_output: Output<SafePollReply>,
    pub(crate) truth_output: Output<TruthRecord>,
    safety_version: u64,
    last_seen: u64
}

impl SafePollingPlatform {
    pub fn new(name: String, initial_value: Value) -> SafePollingPlatform {
        SafePollingPlatform {
            name,
            current_value: initial_value,
            reply_output: Default::default(),
            truth_output: Default::default(),
            safety_version: 0,
            last_seen: 0
        }
    }

    // Input handler for safe platform.
    pub async fn input(&mut self, query: PlatformQuery, ctx: &mut Context<Self>) {
        ctx.schedule_event(ctx.time() + Duration::from_secs(1), Self::process_query, query).unwrap();
    }

    pub async fn process_query(&mut self, query: PlatformQuery, ctx: &mut Context<SafePollingPlatform>) {
        match query {
            PlatformQuery::User(user_action) => match user_action {
                // When user triggered a mutation...
                UserAction::Mutation(delta) => {
                    // info!("{} got {query:?} at {:?}", self.name, ctx.time());
                    self.current_value += delta;
                    // Log to truth out.
                    self.truth_output.send((DefinitionPredicate::AllMut(delta), ctx.time())).await;
                    self.safety_version += 1;
                }
                // When user triggered an assignment...
                UserAction::Assignment(value) => {
                    // info!("{} got {query:?} at {:?}", self.name, ctx.time());
                    self.current_value = value;
                    // Log to truth out.
                    self.truth_output.send((DefinitionPredicate::LastAssn(value), ctx.time())).await;
                    self.safety_version += 1;
                }
            },
            // If interface query...
            PlatformQuery::Interface(InterfaceQuery::PollingSafe(safe_query)) => match safe_query {
                SafePollQuery::Query => {
                    // When getting a query- reply with current state.
                    self.reply_output.send(SafePollReply::Query(self.current_value.clone())).await;
                    self.last_seen = self.safety_version;
                },
                SafePollQuery::Write(to_write, _) => if self.last_seen == self.safety_version {
                    // info!("{} got {query:?} at {:?}", self.name, ctx.time());
                    self.current_value = to_write;
                    // Do write and send success.
                    self.reply_output.send(SafePollReply::WriteSuccess).await;
                    self.safety_version = self.safety_version + 1;
                    self.last_seen = self.safety_version;
                } else {
                    // info!("Write Refused, Last Saw: {}, Current: {}", self.last_seen, self.safety_version);
                    // info!("{} got {query:?} at {:?}", self.name, ctx.time());
                    // Do write failure and send failure.
                    self.reply_output.send(SafePollReply::WriteFail(self.current_value.clone())).await;
                    self.last_seen = self.safety_version;
                }
            },
            x => panic!("Unexpected query type! {x:#?}")
        }
    }
}

impl Model for SafePollingPlatform {}