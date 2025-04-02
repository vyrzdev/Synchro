use log::debug;
use nexosim::model::{Context, Model};
use nexosim::ports::Output;
use crate::simulation::messages::{InterfaceQuery, PlatformQuery, UserAction};
use crate::predicates::DefinitionPredicate;
use crate::simulation::driver::TruthRecord;
use crate::simulation::polling::safe::messages::{SafePollQuery, SafePollReply};
use crate::value::Value;

pub struct SafePollingPlatform {
    name: String,
    current_value: Value,
    pub(crate) reply_output: Output<SafePollReply>,
    pub(crate) truth_output: Output<TruthRecord>
}

impl SafePollingPlatform {
    pub fn new(name: String, initial_value: Value) -> SafePollingPlatform {
        SafePollingPlatform {
            name,
            current_value: initial_value,
            reply_output: Default::default(),
            truth_output: Default::default()
        }
    }

    // Input handler for safe platform.
    pub async fn input(&mut self, query: PlatformQuery, ctx: &mut Context<Self>) {
        match query {
            PlatformQuery::User(user_action) => match user_action {
                // When user triggered a mutation...
                UserAction::Mutation(delta) => {
                    debug!("{} got {query:?} at {:?}", self.name, ctx.time());
                    self.current_value += delta;
                    // Log to truth out.
                    self.truth_output.send((DefinitionPredicate::AllMut(delta), ctx.time())).await;
                }
                // When user triggered an assignment...
                UserAction::Assignment(value) => {
                    debug!("{} got {query:?} at {:?}", self.name, ctx.time());
                    self.current_value = value;
                    // Log to truth out.
                    self.truth_output.send((DefinitionPredicate::LastAssn(value), ctx.time())).await;
                }
            },
            // If interface query...
            PlatformQuery::Interface(InterfaceQuery::PollingSafe(safe_query)) => match safe_query {
                SafePollQuery::Query => {
                    // When getting a query- reply with current state.
                    self.reply_output.send(SafePollReply::Query(self.current_value.clone())).await;
                },
                SafePollQuery::Write(to_write, guard) => if self.current_value == guard {
                    debug!("{} got {query:?} at {:?}", self.name, ctx.time());
                    self.current_value = to_write;
                    // Do write and send success.
                    self.reply_output.send(SafePollReply::WriteSuccess).await;
                } else {
                    debug!("{} got {query:?} at {:?}", self.name, ctx.time());
                    // Do write failure and send failure.
                    self.reply_output.send(SafePollReply::WriteFail(self.current_value.clone())).await;
                }
            },
            x => panic!("Unexpected query type! {x:#?}")
        }

    }
}

impl Model for SafePollingPlatform {}