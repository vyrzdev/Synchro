use std::collections::HashMap;
use std::time::Duration;
use tai_time::MonotonicTime;
use crate::simulation::polling::r#unsafe::UnsafePollingConfig;
use crate::simulation::polling::safe::SafePollingConfig;
use crate::simulation::record::RecordConfig;
use crate::value::Value;

pub struct SimulationConfig {
    pub(crate) initial_value: Value,
    pub(crate) until: MonotonicTime,
    pub(crate) max_divergence_before_error: Duration,
    pub(crate) platforms: HashMap<String, PlatformConfig>,
    // TODO: Record Platforms.
}

pub enum PlatformConfig {
    PollingSafe(SafePollingConfig),
    PollingUnsafe(UnsafePollingConfig),
    Record(RecordConfig),
}
