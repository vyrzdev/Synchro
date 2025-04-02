use std::time::Duration;
use log::info;
use nexosim::model::{Context, InitializedModel, Model};
use nexosim::ports::Output;
use nexosim::simulation::ActionKey;
use nexosim::time::MonotonicTime;
use crate::interpreter::error::ConflictError;
use crate::intervals::Interval;
use crate::observations::{Observation};
use crate::ordering::PlatformMetadata;
use crate::predicates::DefinitionPredicate::{AllMut, LastAssn, Transition};
use crate::simulation::data::SimulationMetaData;
use crate::simulation::messages::{InterfaceQuery, PlatformQuery};
use crate::simulation::polling::config::{PollingInterfaceParameters, PollingInterpretation};
use crate::simulation::polling::data::{FinishedPoll, PollState, SentPoll, WriteState};
use crate::simulation::polling::r#unsafe::messages::{UnsafePollQuery, UnsafePollReply};
use crate::simulation::polling::safe::messages::{SafePollQuery, SafePollReply};
use crate::value::Value;


pub struct UnsafePollingInterface {
    name: String,
    config: PollingInterfaceParameters,
    poll_state: PollState,
    write_state: Option<WriteState>,
    waiting_write: Option<Value>,
    next_scheduled_poll: Option<ActionKey>,
    pub(crate) query_output: Output<PlatformQuery>,
    pub(crate) observation_output: Output<Observation<MonotonicTime>>
}

impl UnsafePollingInterface {
    pub fn new(name: String, config: PollingInterfaceParameters, initial_value: Value) -> UnsafePollingInterface {
        UnsafePollingInterface {
            name,
            config,
            poll_state: PollState {
                ordering: 0,
                current: None,
                last: FinishedPoll {
                    sent: MonotonicTime::EPOCH, // Initial value is from 0.
                    value: initial_value
                }
            },
            write_state: None,
            waiting_write: None,
            next_scheduled_poll: None,
            query_output: Default::default(),
            observation_output: Default::default(),
        }
    }

    pub async fn interpreter_input(&mut self, observed_value: Result<Value, ConflictError<MonotonicTime>>, ctx: &mut Context<Self>) {
        match observed_value {
            // Queue write for after next poll- if write is not yet in flight, overwrite it.
            // Do not cancel poll - minimises unsafe time in exchange for longer convergence.
            Ok(value) => {
                self.waiting_write = Some(value);
            },
            // Do not write on conflict! (Simulation will end soon)
            Err(conflict) => return
        }
    }

    pub async fn platform_input(&mut self, reply: UnsafePollReply, ctx: &mut Context<Self>) {
        match reply {
            // If poll query replies, and does not equal last.
            UnsafePollReply::Query(v) => {
                if v != self.poll_state.last.value {
                    self.generate_observation(self.poll_state.last.value, v, ctx).await;
                }

                let current_poll = self.poll_state.current.take().unwrap();

                // Write last value.
                self.poll_state.last = FinishedPoll {
                    sent: current_poll.at,
                    value: v,
                };

                // If a write is waiting- send it.
                if let Some(waiting) = self.waiting_write {
                    // Send it, guarded with new observed value.
                    self.query_output.send(PlatformQuery::Interface(InterfaceQuery::PollingUnsafe(UnsafePollQuery::Write(waiting)))).await;

                    // Log write.
                    self.write_state = Some(WriteState {
                        sent: ctx.time(),
                        value: waiting,
                    });

                    // Clear waiting.
                    self.waiting_write = None;
                } else {
                    // Otherwise, reschedule poll.
                    self.next_scheduled_poll = Some(
                        // Schedule next poll and save key.
                        ctx.schedule_keyed_event(ctx.time() + self.config.backoff, Self::poll, ()).unwrap() // Always in future.
                    );
                }
            },
            // If write is complete...
            UnsafePollReply::WriteComplete => {
                let successful_write = self.write_state.take().unwrap();

                // Then written value is now last-
                self.poll_state.last = FinishedPoll {
                    value: successful_write.value,
                    sent: successful_write.sent,
                };

                // Only send waiting writes on receipt of poll, sacrifices convergence time for improved safety.
                // Therefore, schedule next poll
                self.next_scheduled_poll = Some(
                    // Schedule next poll and save key.
                    ctx.schedule_keyed_event(ctx.time() + self.config.backoff, Self::poll, ()).unwrap() // Always in future.
                );
            }
        }
    }

    pub async fn generate_observation(&mut self, from: Value, to: Value, ctx: &mut Context<Self>) {
        self.observation_output.send(Observation {
            interval: Interval(self.poll_state.last.sent, ctx.time()),
            definition_predicate: match self.config.interp {
                PollingInterpretation::Transition => Transition(from, to),
                PollingInterpretation::AllMut => AllMut(to - from),
                PollingInterpretation::LastAssn => LastAssn(to)
            },
            source: self.name.clone(),
            platform_metadata: PlatformMetadata::Simulation(SimulationMetaData {
                monotonic: self.poll_state.ordering
            }),
        }).await;
        self.poll_state.ordering += 1;
    }

    pub async fn poll(&mut self, _: (), ctx: &mut Context<Self>) {
        // Log poll.
        self.poll_state.current = Some(SentPoll {
            at: ctx.time(),
        });
        // Send query.
        self.query_output.send(PlatformQuery::Interface(InterfaceQuery::PollingUnsafe(UnsafePollQuery::Query))).await;
        self.next_scheduled_poll = None; // Just did it!
    }
}
impl Model for UnsafePollingInterface {
    async fn init(mut self, ctx: &mut Context<Self>) -> InitializedModel<Self> {
        self.next_scheduled_poll = Some(
            // Schedule next poll and save key.
            ctx.schedule_keyed_event(ctx.time() + self.config.backoff, Self::poll, ()).unwrap() // Always in future.
        );
        self.into()
    }
}