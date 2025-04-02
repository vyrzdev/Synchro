use std::time::Duration;
use crate::simulation::polling::r#unsafe::UnsafePollingConfig;
use crate::simulation::polling::safe::SafePollingConfig;
// Common interface params for Unsafe and Safe


#[derive(Clone, Debug)]
pub struct PollingInterfaceParameters {
    pub(crate) interp: PollingInterpretation,
    pub(crate) backoff: Duration,
}

#[derive(Clone, Debug)]
pub enum PollingInterpretation {
    Transition,
    AllMut,
    LastAssn
}