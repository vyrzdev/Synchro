use std::time::Duration;
use crate::value::Value;

pub mod user;

#[derive(Clone, Debug)]
pub struct UserParameters {
    pub(crate) average_sales_per_hour: f64,
    pub(crate) average_edits_per_day: f64,
    pub(crate) edit_to: Value,
    pub(crate) start_after: Duration
}