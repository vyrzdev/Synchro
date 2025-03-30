use std::time::Duration;
use nexosim::model::{Context, InitializedModel, Model};
use nexosim::ports::Output;
use nexosim::time::MonotonicTime;
use crate::messages;
use crate::messages::{PollReply, PollRequest, Write};
use crate::observations::{Observation, PlatformMetadata};
use crate::observations::DefinitionPredicate::{AllMut, LastAssn, Transition};
use crate::polling::PollingInterpretation;
use crate::value::Value;

pub struct WaitingPoll {
    send: MonotonicTime
}

pub struct CompletedPoll {
    send: MonotonicTime,
    receive: MonotonicTime,
    value: Value
}
pub struct WaitingWrite {
    send: MonotonicTime,
    value: Value
}

pub struct UnsafePollingInterface {
    interp: PollingInterpretation,
    pub(crate) request_output: Output<PollRequest>,
    pub(crate) observation_output: Output<messages::Observation>,
    backoff: Duration,
    current_poll: Option<WaitingPoll>,
    current_write: Option<WaitingWrite>,
    last: Option<CompletedPoll>,
    to_write: Option<Value>,
    poll_count: u64,
    name: String
}
impl UnsafePollingInterface {
    pub fn new(name: String, backoff: Duration, interp: PollingInterpretation) -> UnsafePollingInterface {
        UnsafePollingInterface {
            name,
            interp,
            request_output: Output::default(),
            observation_output: Output::default(),
            backoff,
            current_poll: None,
            current_write: None,
            last: None,
            to_write: None,
            poll_count: 0
        }
    }

    pub fn write_input(&mut self, write: Write, ctx: &mut Context<Self>) {
        // In unsafe-write mode; we write on receiving a poll, to minimise potential unsafe time*
        match self.to_write.as_mut() {
            None => {
                self.to_write = Some(write.value); // No Write Session Yet - Add Write!
            },
            Some(v) => {
                *v = write.value; // Override- We have changed value.
            }
        }
    }

    pub async fn reply_input(&mut self, reply: PollReply, ctx: &mut Context<Self>) {
        match reply {
            PollReply::Query(value) => {
                if self.last.as_ref().is_some_and(|last| last.value != value) {
                    // Generate Appropriate Observation.
                    self.observation_output.send(messages::Observation{
                        observation:Observation {
                            interval: (self.last.as_ref().unwrap().send, ctx.time()),
                            definition_predicate: match self.interp {
                                PollingInterpretation::Transition => Transition(
                                    self.last.as_ref().unwrap().value,
                                    value
                                ),
                                PollingInterpretation::AllMut => AllMut(
                                    value - self.last.as_ref().unwrap().value
                                ),
                                PollingInterpretation::LastAssn => LastAssn(
                                    value
                                )
                            },
                            source: self.name.clone(),
                            platform_metadata: PlatformMetadata::Polling {
                                poll_count: self.poll_count
                            },
                        }
                    }).await;
                    self.poll_count += 1; // Monotonic ordering relation.
                }

                // Should not get poll reply when one is not sent!
                // Clear historic poll - add new record.
                let current =  self.current_poll.take().unwrap();
                self.last = Some(CompletedPoll {
                    send: current.send,
                    receive: ctx.time(),
                    value
                });


                // Increment poll count (monotonic ordering relation) (\prec_R)
                self.poll_count += 1;

                match self.to_write.as_ref() {
                    Some(value) => {
                        // If we have a value to write... send it!
                        self.request_output.send(PollRequest::Write(*value)).await;
                        // And log send time, and value written
                        self.current_write = Some(WaitingWrite {
                            send: ctx.time(),
                            value: value.clone()
                        });
                        // And clear to_write (no duplicates)
                        self.to_write = None;
                    },
                    None => {
                        // If no write pending- do backoff!
                        ctx.schedule_event(self.backoff, Self::poll, (ctx.time())).unwrap()
                    }
                }
            }
            PollReply::WriteComplete => {
                // Override last poll with write (psuedo poll), expect current!
                let current = self.current_write.as_ref().unwrap();

                self.last = Some(CompletedPoll {
                    send: current.send,
                    receive: ctx.time(),
                    value: current.value,
                });
                // Clear this write session (its complete)
                self.current_write = None;
                // Write is completed- Reschedule poll!
                ctx.schedule_event(self.backoff, Self::poll, (ctx.time())).unwrap();
            }
        }
    }

    pub async fn poll(&mut self, now: MonotonicTime) {
        self.current_poll = Some(WaitingPoll { send: now });
        self.request_output.send(PollRequest::Query {}).await;
    }
}
impl Model for UnsafePollingInterface {
    async fn init(mut self, ctx: &mut Context<Self>) -> InitializedModel<Self> {
        self.poll(ctx.time()).await;
        self.into()
    }
}