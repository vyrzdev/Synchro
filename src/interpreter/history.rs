use std::collections::LinkedList;
use std::fmt::Debug;
use log::debug;
use crate::interpreter::error::ConflictError;
use crate::interpreter::regions::Region;
use crate::value::Value;

#[derive(Debug)]
pub struct History<T: PartialOrd + Clone>(pub(crate) LinkedList<Region<T>>);

impl<T: PartialOrd + Clone + Debug> History<T> {
    pub fn new() -> Self {
        Self(LinkedList::new())
    }

    pub fn apply(&mut self, mut value: Option<Value>, at: T) -> Result<Value, ConflictError<T>> {
        // Track where conflict happened.
        let mut conflict_region = None;

        // Iterate over each region.
        for mut region in &mut self.0 {
            // Attempt to apply.
            let result = region.apply(value);

            // If this is the root of the conflict- i.e. where it occurred. Then track.
            if result.is_none() && conflict_region.is_none() {
                conflict_region.replace(region);
            }

            // If a result is defined- clear conflict region.
            if result.is_some() {
                conflict_region = None;
            }
            value = result;
        }

        match value {
            Some(v) => Ok(v),
            None => Err(ConflictError::<T> {
                reason: "Conflict due to change!".to_string(),
                observations: conflict_region.unwrap().observations.clone(),
                at
            })
        }
    }
}