use log::{error};
use nexosim::model::{Context, InitializedModel, Model};
use nexosim::ports::Output;
use nexosim::simulation::ActionKey;
use nexosim::time::MonotonicTime;
use crate::core::interpreter::error::ConflictError;
use crate::core::intervals::Interval;
use crate::core::observations::Observation;
use crate::core::ordering::PlatformMetadata;
use crate::core::predicates::DefinitionPredicate::{AllMut, LastAssn, Transition};
use crate::simulation::data::SimulationMetaData;
use crate::simulation::messages::{InterfaceQuery, PlatformQuery};
use crate::simulation::polling::config::{PollingInterfaceParameters, PollingInterpretation};
use crate::simulation::polling::data::{FinishedPoll, PollState, SentPoll, WriteState};
use crate::simulation::polling::safe::messages::{SafePollQuery, SafePollReply};
use crate::value::Value;


pub struct SafePollingInterface {
    name: String,
    config: PollingInterfaceParameters,
    poll_state: PollState,
    write_state: Option<WriteState>,
    waiting_write: Option<Value>,
    next_scheduled_poll: Option<ActionKey>,
    pub(crate) query_output: Output<PlatformQuery>,
    pub(crate) observation_output: Output<Observation<MonotonicTime>>
}

impl SafePollingInterface {
    pub fn new(name: String, config: PollingInterfaceParameters, initial_value: Value) -> SafePollingInterface {
        SafePollingInterface {
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
            Ok(value) => {
                // Take next poll schedule, and leave none in its place.
                let next_poll = self.next_scheduled_poll.take();
                if let Some(next_poll) = next_poll {
                    // If there was a next poll, CANCEL IT.
                    next_poll.cancel();
                    self.next_scheduled_poll = None;

                    // And send the write now.
                    // If no write has been sent- clear any waiting and send.
                    self.waiting_write = None;
                    // Guard with last observed value.
                    self.query_output.send(PlatformQuery::Interface(InterfaceQuery::PollingSafe(SafePollQuery::Write(value, self.poll_state.last.value)))).await;
                    // Log write.
                    // info!("TRACE3 - WRITE STATE SET TO {}", ctx.time());
                    self.write_state = Some(WriteState {
                        sent: ctx.time(),
                        value
                    });
                // If there is no next poll, either a different write has been sent or poll is inflight.
                // Therefore, wait to send write until replies.
                } else {
                    self.waiting_write = Some(value);
                }
            },
            // Do not write on conflict! (Simulation will end soon)
            Err(_) => return
        }
    }

    pub async fn platform_input(&mut self, reply: SafePollReply, ctx: &mut Context<Self>) {
        // info!("{} got {reply:?} at {:?}", self.name, ctx.time());
        match reply {
            // If poll query replies, and does not equal last.
            SafePollReply::Query(v) => {
                let current_poll = self.poll_state.current.take().unwrap();

                // info!{"Current Poll: {}", current_poll.at}
                if v != self.poll_state.last.value {
                    // info!("Generating Observation at {:?}", ctx.time());
                    self.generate_observation(self.poll_state.last.value, v, ctx).await;
                }


                // Write last value.
                // info!("Set Poll State To: {:?} at: {:?}", current_poll.at, ctx.time());
                self.poll_state.last = FinishedPoll {
                    sent: current_poll.at,
                    value: v,
                };

                // If a write is waiting- send it.
                if let Some(waiting) = self.waiting_write {
                    // debug!("Write waiting on inflight poll- being sent now!");
                    // Send it, guarded with new observed value.
                    self.query_output.send(PlatformQuery::Interface(InterfaceQuery::PollingSafe(SafePollQuery::Write(waiting, v)))).await;

                    // Log write.
                    // info!("TRACE2 - WRITE STATE SET TO {}", ctx.time());
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
            // If write was successful...
            SafePollReply::WriteSuccess => {
                let successful_write = self.write_state.take().unwrap();

                // Then written value is now last-
                self.poll_state.last = FinishedPoll {
                    value: successful_write.value,
                    sent: successful_write.sent,
                };

                // If there's another write waiting...
                if let Some(waiting) = self.waiting_write {
                    // debug!("Write waiting on inflight write- being sent now!");
                    // Send it, guarded with new successfully written value.
                    self.query_output.send(PlatformQuery::Interface(InterfaceQuery::PollingSafe(SafePollQuery::Write(waiting, successful_write.value)))).await;

                    // info!("TRACE1 - WRITE STATE SET TO {}", ctx.time());
                    // Log write.
                    self.write_state = Some(WriteState {
                        sent: ctx.time(),
                        value: waiting,
                    });

                    // Clear waiting.
                    self.waiting_write = None;
                } else {
                    // Otherwise, schedule the next poll...
                    self.next_scheduled_poll = Some(
                        // Schedule next poll and save key.
                        ctx.schedule_keyed_event(ctx.time() + self.config.backoff, Self::poll, ()).unwrap() // Always in future.
                    );
                }
            }
            // If write failed...
            SafePollReply::WriteFail(new_value) => {

                // debug!("Write failed- voiding pending write and making observation!");
                let failed_write = self.write_state.take().unwrap();
                // Is due to a change-
                // info!("WRITE FAILED: {:?}, {:?}, {:?}, {:?}, {:?}", failed_write, self.poll_state.last.value, new_value, self.poll_state.last.sent, ctx.time());
                self.generate_observation(self.poll_state.last.value, new_value, ctx).await;

                // If write waiting, clear it as new obs will likely overwrite it.
                self.waiting_write = None;

                // Log seen new-value as last.
                self.poll_state.last = FinishedPoll {
                    value: new_value,
                    sent: failed_write.sent,
                };

                // Schedule new poll- for the hell of it.
                // It will probably get overwritten by the next, but it would be nice to get action coverage over processing time.
                self.next_scheduled_poll = Some(
                    // Schedule next poll and save key.
                    ctx.schedule_keyed_event(ctx.time() + self.config.backoff, Self::poll, ()).unwrap() // Always in future.
                );
            }
        }
    }

    pub async fn generate_observation(&mut self, from: Value, to: Value, ctx: &mut Context<Self>) {
        if ctx.time() < self.poll_state.last.sent {
            error!("Got Backwards Observation: {:?}, {:?}", self.poll_state.last.sent, ctx.time());
        };
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
        // if self.name=="Polling1" {
        //     // debug!("Polled At: {}", ctx.time());
        // }
        // info!("Wrote Current Poll to {}", ctx.time());
        // Log poll.
        self.poll_state.current = Some(SentPoll {
            at: ctx.time(),
        });
        // Send query.
        self.query_output.send(PlatformQuery::Interface(InterfaceQuery::PollingSafe(SafePollQuery::Query))).await;
        self.next_scheduled_poll = None; // Just did it!
    }
}
impl Model for SafePollingInterface {
    async fn init(mut self, ctx: &mut Context<Self>) -> InitializedModel<Self> {
        self.next_scheduled_poll = Some(
            // Schedule next poll and save key.
            ctx.schedule_keyed_event(ctx.time() + self.config.backoff, Self::poll, ()).unwrap() // Always in future.
        );
        self.into()
    }
}