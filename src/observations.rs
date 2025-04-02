use std::fmt::{Display, Formatter};
use tai_time::MonotonicTime;
use crate::intervals::Interval;
use crate::ordering::PlatformMetadata;
use crate::predicates::DefinitionPredicate;

#[derive(Clone, Debug)]
pub struct Observation<T: PartialOrd + Clone> {
    pub(crate) interval: Interval<T>, // Uncertainty Interval, generic over time-type, see interval.rs
    pub(crate) definition_predicate: DefinitionPredicate, // Definition Predicate, see predicates.rs
    pub(crate) source: String, // Source Platform Identifier (unique for each)
    pub(crate) platform_metadata: PlatformMetadata // Platform-Level Ordering, see ordering.rs
}

impl<T: PartialOrd + Clone> Display for Observation<T> {
    // Debug/Display output :)
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", match self.definition_predicate {
            DefinitionPredicate::AllMut(_) => "MU",
            DefinitionPredicate::Transition(_, _) => "TR",
            DefinitionPredicate::LastAssn(_) => "AS",
            DefinitionPredicate::Unknown => "UK"
        },
            self.source,

            // Display from beginning of time!
            // self.interval.0.duration_since(MonotonicTime::EPOCH).as_millis(),
            // self.interval.1.duration_since(MonotonicTime::EPOCH).as_millis()
        )
    }
}