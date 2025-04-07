use serde::{Deserialize, Serialize};
use crate::real_world::square::polling::SquarePollingConfig;
use crate::real_world::square::record::SquareRecordConfig;
use crate::value::Value;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RealWorldConfig {
    /// Initial value to write to platforms
    pub(crate) initial_value: Value,
    /// Platforms to use.
    pub(crate) platforms: Vec<(String,PlatformConfig)>,
}

/// Configuration for Platforms
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlatformConfig {
    /// Polling Platform Config
    Polling(SquarePollingConfig),
    /// Record Platform Config
    Records(SquareRecordConfig)
}