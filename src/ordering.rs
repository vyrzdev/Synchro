use std::cmp::Ordering;
use crate::observations::Observation;
use crate::simulation::data::SimulationMetaData;
use crate::real_world::square::SquareMetadata;

#[derive(Clone, Debug)]
pub enum PlatformMetadata {
    Square(SquareMetadata), // Square Logical Ordering
    Simulation(SimulationMetaData) // Simulation Logical Ordering
}

impl PartialEq<Self> for PlatformMetadata {
    fn eq(&self, _: &Self) -> bool {
        false // Events are unique!
    }
}

impl PartialOrd for PlatformMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            // When platforms are both square, defer to square ordering.
            (PlatformMetadata::Square(a), PlatformMetadata::Square(b)) => match a.partial_cmp(b) {
                Some(Ordering::Equal) => None,
                x => x
            },
            // When platforms are both simulation, defer to simulation ordering.
            (PlatformMetadata::Simulation(a), PlatformMetadata::Simulation(b)) => match a.partial_cmp(b) {
                Some(Ordering::Equal) => None,
                x => x
            },
            _ => None // Platforms cannot be ordered.
        }
        // NOTE: Data typing only, we check if a.source==b.source before trusting this ordering.
    }
}

impl<T: PartialOrd + Clone> PartialEq for Observation<T> {
    fn eq(&self, _: &Self) -> bool {
        false // Observations are UNIQUE.
    }
}

impl<T: PartialOrd + Clone> PartialOrd for Observation<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.interval.partial_cmp(&other.interval) {
            // If intervals overlap, but from same replica, check source logical ordering.
            None if self.source == other.source => self.platform_metadata.partial_cmp(&other.platform_metadata),
            x => x // Otherwise, return interval ordering.
        }
    }
}