use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use nexosim::time::MonotonicTime;
use crate::value::Value;
use crate::intervals::{Interval, GT, LT};


#[derive(Clone, Debug)]
pub struct Observation {
    pub(crate) interval: Interval,
    pub(crate) definition_predicate: DefinitionPredicate,
    pub(crate) source: String, // Source Platform.
    pub(crate) platform_metadata: PlatformMetadata // TODO: Ordering Relation.
}

#[derive(Clone, Debug)]
pub enum DefinitionPredicate {
    Transition(Value, Value),
    AllMut(Value),
    LastAssn(Value)
}

#[derive(Clone, Debug)]
pub enum PlatformMetadata {
    Polling {
        poll_count: u64 // Monotonic platform-local ordering relation.
    },
    Record {
        logical_version: u64 // Monotonic platform-local ordering relation.
    }
}

impl PartialEq for Observation {
    fn eq(&self, other: &Self) -> bool {
        false // Observations are UNIQUE.
    }
}

impl PartialOrd for Observation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // prec_O - Observed ordering relation.
        if LT(self.interval, other.interval) {
            Some(Ordering::Less) // self < other.
        } else if GT(self.interval, other.interval) {
            Some(Ordering::Greater) // self > other
        } else {
            if self.source == other.source {
                match self.platform_metadata {
                    PlatformMetadata::Polling { poll_count } => {
                        let PlatformMetadata::Polling { poll_count: other_poll_count } = other.platform_metadata else {panic!("Should Be Equivalent!")};
                        return Some(poll_count.cmp(&other_poll_count));
                    },
                    PlatformMetadata::Record { logical_version } => {
                        // TODO: Platform-Level Ordering Relations
                        None
                    }
                }

            } else {
                None
            }
        }
    }
}

impl Display for Observation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} @ {}, {}", match self.definition_predicate {
            DefinitionPredicate::AllMut(_) => "MU",
            DefinitionPredicate::Transition(_, _) => "TR",
            DefinitionPredicate::LastAssn(_) => "AS"
        },
            self.source,
            self.interval.0.duration_since(MonotonicTime::EPOCH).as_millis(),
            self.interval.1.duration_since(MonotonicTime::EPOCH).as_millis()
        )
    }
}