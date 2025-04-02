use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct SimulationMetaData {
    pub(crate) monotonic: u64
}
impl PartialEq for SimulationMetaData {
    fn eq(&self, other: &Self) -> bool {
        false // Monotonics are never equal.
    }
}

impl PartialOrd for SimulationMetaData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.monotonic < other.monotonic {
            Some(Ordering::Less)
        } else if self.monotonic > other.monotonic {
            Some(Ordering::Greater)
        } else {
            None
        }
    }
}