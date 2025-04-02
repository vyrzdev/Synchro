use crate::observations::Observation;

// Typed conflict error - common between real and unsafe implementation.
#[derive(Debug, Clone)]
pub struct ConflictError<T: PartialOrd + Clone> {
    pub(crate) reason: String,
    pub(crate) at: T,
    pub(crate) observations: Vec<Observation<T>>
}
