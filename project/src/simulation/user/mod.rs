use std::time::Duration;
use serde::{Deserialize, Serialize};
use tai_time::MonotonicTime;
use crate::value::Value;
use crate::simulation::config::serde_monotonic_helper;

pub mod user;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserParameters {
    pub(crate) average_sales_per_hour: f64,
    pub(crate) average_edits_per_day: f64,
    pub(crate) edit_to: Value,
    pub(crate) start_after: Duration,
    #[serde(with="serde_monotonic_helper")]
    pub(crate) until: MonotonicTime
}