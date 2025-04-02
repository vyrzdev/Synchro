use crate::real_world::square::polling::SquarePollingConfig;
use crate::real_world::square::record::SquareRecordConfig;
use crate::value::Value;


pub struct RealWorldConfig {
    pub(crate) platforms: Vec<(String,PlatformConfig)>,
    pub(crate) initial_value: Value,
}

pub enum PlatformConfig {
    Polling(SquarePollingConfig),
    Records(SquareRecordConfig)
}