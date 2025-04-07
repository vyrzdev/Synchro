use std::collections::HashMap;
use std::path::{Path, PathBuf};
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

/// Generates an output path for logging based on the given output folder
/// and input path. If the generated path already exists, it appends a
/// numeric suffix to ensure the path is unique.
///
/// # Arguments
///
/// * `output_folder` - The folder where the output file should be stored.
/// * `input_path` - The original input file path used to derive the output path.
///
/// # Returns
///
/// A unique output `PathBuf`.
pub fn output_path(output_folder: &PathBuf, input_path: &PathBuf) -> PathBuf {
    let candidate = output_folder.join(input_path.file_stem().unwrap()).with_extension("log");
    return resolve_unique_path(&candidate);
}

/// Resolves a unique file path by checking if the given `original` path exists.
/// If it exists, appends a numeric suffix to the file name to ensure uniqueness.
///
/// # Arguments
///
/// * `original` - The original file path to be evaluated.
///
/// # Returns
///
/// A `PathBuf` representing a unique file path.
///
/// # Example
/// ```
/// let original = Path::new("output.log");
/// let unique_path = resolve_unique_path(original);
/// println!("{:?}", unique_path); // Could be "output_1.log" if "output.log" exists.
/// ```
pub fn resolve_unique_path(original: &Path) -> PathBuf {
    if !original.exists() { // if file dont exist- return it.
        return original.to_path_buf();
    }

    // otherwise... exists
    let stem = original.file_stem().unwrap_or_default().to_string_lossy(); // get stem
    let ext = original.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap(); // get extension and add
    let parent = original.parent().unwrap_or_else(|| Path::new(""));//

    for i in 1.. {
        let candidate = parent.join(format!("{stem}_{i}{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!() // Compile please! (try taking it away)
}

/// Macro to quickly create test observations.
///
/// # Usage
/// ```rust
/// let observations = vec![
///     make_observation!(0, 10),
///     make_observation!(25, 65),
/// ];
/// ```
///
/// # Arguments
/// * `$start` - Start time for the observation (in seconds).
/// * `$end` - End time for the observation (in seconds).
macro_rules! make_observation {

    ($start:expr, $end:expr) => {
        Observation {
            interval: (
                MonotonicTime::new($start, 0).unwrap(),
                MonotonicTime::new($end, 0).unwrap(),
            ),
            definition_predicate: DefinitionPredicate::Unknown,
            source: "".to_string(),
            platform_metadata: PlatformMetadata::Polling { poll_count: 0 },
        }
    };
}

/// Generates an initial simulation config (for default json)
pub fn make_demo_sim() -> SimulationConfig {
    SimulationConfig {
        initial_value: 100,
        until: MonotonicTime::new(10000, 0).unwrap(),
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