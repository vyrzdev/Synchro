use chrono::{DateTime, Utc};

pub mod polling;
pub mod record;

pub const IGNORE: &str = "IGNORE";
pub type Target = (String, String);


#[derive(PartialEq, PartialOrd, Copy, Clone, Debug)]
pub struct SquareMetadata {
    pub(crate) timestamp: DateTime<Utc>
}
