use std::cmp::Ordering;
#[derive(Debug, Copy, Clone)]
pub struct Interval<T: PartialOrd>(pub T, pub T); // Generic Interval Type.

impl<T: PartialOrd> PartialEq for Interval<T> {
    fn eq(&self, _other: &Self) -> bool {
        false // When intervals cannot be ordered - we take it as UNDEFINED!
    }
}

impl<T: PartialOrd> PartialOrd for Interval<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.1 < other.0 { // If a_{end} < b_{start}
            Some(Ordering::Less) // a < b
        } else if self.0 > other.1 { // If a_{start} > b_{end}
            Some(Ordering::Greater) // a > b
        } else {
            None // Otherwise, intervals overlap and cannot be ordered!
        }
    }
}

// Chrono Intervals, ideal for UTC calculations and safely decoding Square representation.
// pub type RealIntervals = Interval<DateTime<Utc>>;
// Simulator requires use of dedicated time-type, MonotonicTime.
// pub type SimulationIntervals = Interval<MonotonicTime>;