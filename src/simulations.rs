use std::collections::HashMap;
use std::time::Duration;
use chrono::TimeDelta;
use tai_time::MonotonicTime;
use crate::simulation::config::{PlatformConfig, SimulationConfig};
use crate::simulation::network::network_delay::NetworkParameters;
use crate::simulation::polling::config::{PollingInterfaceParameters, PollingInterpretation};
use crate::simulation::polling::r#unsafe::UnsafePollingConfig;
use crate::simulation::polling::safe::SafePollingConfig;
use crate::simulation::record::interface::RecordInterfaceParameters;
use crate::simulation::record::platform::RecordPlatformParameters;
use crate::simulation::record::RecordConfig;
use crate::simulation::user::UserParameters;

pub fn make_demo_sim() -> SimulationConfig {
    SimulationConfig {
        initial_value: 100,
        until: MonotonicTime::new(10000, 0).unwrap(),
        max_divergence_before_error: Duration::new(1, 0),
        platforms: HashMap::from([
            ("Polling1".to_string(), PlatformConfig::PollingUnsafe(UnsafePollingConfig {
                initial_value: 100,
                network_params: NetworkParameters {
                    size: 40.0,
                    scale: 4.0,
                },
                interface_params: PollingInterfaceParameters {
                    interp: PollingInterpretation::Transition,
                    backoff: Duration::from_millis(200),
                },
                user_params: UserParameters {
                    until: MonotonicTime::new(100000, 0).unwrap(),
                    average_sales_per_hour: 2.0,
                    average_edits_per_day: 1.0,
                    edit_to: 100,
                    start_after: Default::default(),
                },
            })),
            ("Polling2".to_string(), PlatformConfig::PollingSafe(SafePollingConfig {
                initial_value: 100,
                network_params: NetworkParameters {
                    size: 20.0,
                    scale: 2.0,
                },
                interface_params: PollingInterfaceParameters {
                    interp: PollingInterpretation::Transition,
                    backoff: Duration::from_millis(200),
                },
                user_params: UserParameters {
                    until: MonotonicTime::new(100000, 0).unwrap(),
                    average_sales_per_hour: 10.0,
                    average_edits_per_day: 5.0,
                    edit_to: 100,
                    start_after: Default::default(),
                },
            })),
            ("Record1".to_string(), PlatformConfig::Record(RecordConfig {
                network_params: NetworkParameters { size: 40.0, scale: 2.0 },
                interface_params: RecordInterfaceParameters {
                    backoff: Duration::from_millis(200),
                },
                platform_params: RecordPlatformParameters {
                    deviation: TimeDelta::milliseconds(-400),
                },
                user_params: UserParameters {
                    until: MonotonicTime::new(100000, 0).unwrap(),
                    average_sales_per_hour: 100.0,
                    average_edits_per_day: 20.0,
                    edit_to: 100,
                    start_after: Duration::from_millis(1000),
                },
            }))
        ])
    }
}