use std::fmt::{Display, Formatter};
use crate::core::intervals::Interval;
use crate::core::ordering::PlatformMetadata;
use crate::core::predicates::DefinitionPredicate;


/// Represents an observation with an uncertainty interval, definition predicate, source identifier,
/// and platform metadata.
///
/// # Generics
/// - `T`: The type of the time used in the interval. Must implement `PartialOrd` and `Clone`.
///
#[derive(Clone, Debug)]
pub struct Observation<T: PartialOrd + Clone> {
    /// Uncertainty Interval, generic over time-type, see interval.rs
    pub(crate) interval: Interval<T>,
    /// Definition Predicate, see predicates.rs
    pub(crate) definition_predicate: DefinitionPredicate,
    /// Source Platform Identifier (unique for each)
    pub(crate) source: String,
    /// Platform-Level Ordering, see ordering.rs
    pub(crate) platform_metadata: PlatformMetadata,
}

impl<T: PartialOrd + Clone> Display for Observation<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", match self.definition_predicate {
            DefinitionPredicate::AllMut(_) => "MU", // Mutation predicate
            DefinitionPredicate::Transition(_, _) => "TR", // Transition predicate
            DefinitionPredicate::LastAssn(_) => "AS", // Assignment predicate
            DefinitionPredicate::Unknown => "UK" // Unknown predicate
        },
               self.source,
        )
    }
}