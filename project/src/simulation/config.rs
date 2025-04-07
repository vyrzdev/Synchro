use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use tai_time::MonotonicTime;
use crate::simulation::polling::r#unsafe::UnsafePollingConfig;
use crate::simulation::polling::safe::SafePollingConfig;
use crate::simulation::record::RecordConfig;
use crate::value::Value;


/// The `SimulationConfig` struct holds the configuration parameters for a simulation.
///
/// # Fields
/// - `initial_value`: The initial value of the simulation, represented as a `Value`.
/// - `until`: A `MonotonicTime` representing the point in time until which the simulation runs.
///            This field is serialized and deserialized using the `serde_monotonic_helper` module.
/// - `platforms`: A `HashMap` mapping platform names (as `String`) to their respective
///            `PlatformConfig`.
///
/// # Example
/// ```
/// use tai_time::MonotonicTime;
/// use std::collections::HashMap;
/// use std::time::Duration;
/// use crate::{Value, SimulationConfig, PlatformConfig};
///
/// let config = SimulationConfig {
///     initial_value: Value::new(), // Example, replace with the actual `Value` constructor.
///     until: MonotonicTime::now(),
///     platforms: HashMap::new(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub(crate) initial_value: Value,
    #[serde(with="serde_monotonic_helper")]
    pub(crate) until: MonotonicTime,
    pub(crate) platforms: HashMap<String, PlatformConfig>,
}

pub mod serde_monotonic_helper {
    use chrono::DateTime;
    use serde::{Serializer, Deserializer, Deserialize};
    use tai_time::MonotonicTime;

    pub fn serialize<S>(v: &MonotonicTime, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let secs = v.to_chrono_date_time(0).unwrap().timestamp() as i64;
        s.serialize_i64(secs)
    }


    pub fn deserialize<'de, D>(d: D) -> Result<MonotonicTime, D::Error>
    where D: Deserializer<'de> {
        let secs = i64::deserialize(d)?;
        Ok(MonotonicTime::from_chrono_date_time(&DateTime::from_timestamp(secs, 0).unwrap(), 0))
    }
}

/// platform config types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlatformConfig {
    PollingSafe(SafePollingConfig),
    PollingUnsafe(UnsafePollingConfig),
    Record(RecordConfig),
}
