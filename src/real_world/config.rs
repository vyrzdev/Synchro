use serde::{Deserialize, Serialize};
use crate::real_world::square::polling::SquarePollingConfig;
use crate::real_world::square::record::SquareRecordConfig;
use crate::value::Value;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RealWorldConfig {
    pub(crate) initial_value: Value, // Initial value to write to platforms
    pub(crate) platforms: Vec<(String,PlatformConfig)>, // Platforms to use.
}

#[derive(Clone, Debug, Serialize, Deserialize)]

pub enum PlatformConfig {
    Polling(SquarePollingConfig),
    Records(SquareRecordConfig)
}