use std::time::Duration;
use serde::{Deserialize, Serialize};

// Common interface params for Unsafe and Safe


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PollingInterfaceParameters {
    pub(crate) interp: PollingInterpretation,
    pub(crate) backoff: Duration,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PollingInterpretation {
    Transition,
    AllMut,
    LastAssn
}