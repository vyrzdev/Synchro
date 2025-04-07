use nexosim::model::{Context, Model};
use nexosim::ports::Output;
use crate::core::predicates::DefinitionPredicate;
use crate::simulation::messages::{InterfaceQuery, PlatformQuery, UserAction};
use crate::simulation::driver::TruthRecord;
use crate::simulation::polling::r#unsafe::messages::{UnsafePollQuery, UnsafePollReply};
use crate::value::Value;

pub struct UnsafePollingPlatform {
    name: String,
    current_value: Value,
    pub(crate) reply_output: Output<UnsafePollReply>,
    pub(crate) truth_output: Output<TruthRecord>
}

impl UnsafePollingPlatform {
    pub fn new(name: String, initial_value: Value) -> UnsafePollingPlatform {
        UnsafePollingPlatform {
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
                    self.current_value += delta;
                    // Log to truth out.
                    self.truth_output.send((DefinitionPredicate::AllMut(delta), ctx.time())).await;
                }
                // When user triggered an assignment...
                UserAction::Assignment(value) => {
                    self.current_value = value;
                    // Log to truth out.
                    self.truth_output.send((DefinitionPredicate::LastAssn(value), ctx.time())).await;
                }
            },
            // If interface query...
            PlatformQuery::Interface(InterfaceQuery::PollingUnsafe(unsafe_query)) => match unsafe_query {
                UnsafePollQuery::Query => {
                    // When getting a query- reply with current state.
                    self.reply_output.send(UnsafePollReply::Query(self.current_value.clone())).await;
                },
                UnsafePollQuery::Write(to_write) => {
                    self.current_value = to_write;
                    // Do write and send success.
                    self.reply_output.send(UnsafePollReply::WriteComplete).await;
                }
            },
            x => panic!("Unexpected query type! {x:#?}")
        }

    }
}

impl Model for UnsafePollingPlatform {}