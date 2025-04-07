use std::collections::LinkedList;
use std::fmt::Debug;
use Clone;
use log::info;
use crate::core::interpreter::error::ConflictError;
use crate::core::interpreter::regions::Region;
use crate::value::Value;

#[derive(Debug)]
pub struct History<T: PartialOrd + Clone> {
    pub(crate) list: LinkedList<Region<T>>,
}

impl<T: PartialOrd + Clone + Debug> History<T> {
    pub fn new() -> Self {
        Self {
            list: LinkedList::new(),
        }
    }


    /// Applies a value to the history at a specific point in time (`at`).
    /// Iterates over the regions in the history and attempts to apply the value.
    /// If a conflict occurs, it returns a `ConflictError` containing details of the conflict.
    ///
    /// # Parameters
    /// - `value`: The optional `Value` to be applied.
    /// - `at`: The point in time where the value is being applied. (for debug only)
    ///
    /// # Returns
    /// - `Ok(Value)` if the value is successfully applied.
    /// - `Err(ConflictError<T>)` if a conflict is detected.
    ///
    /// # Errors
    /// Returns a `ConflictError` if a conflict arises while applying the value.
    pub fn apply(&mut self, mut value: Option<Value>, at: T) -> Result<Value, ConflictError<T>> {
        // Track where conflict happened.
        let mut conflict_region = None;
        // println!("LIST: {:?}", self.list);
        // Iterate over each region.
        for region in &mut self.list {
            // Attempt to apply.
            let result = region.apply(value);
            // info!("{result:?}, {value:?}");
            // If this is the root of the conflict- i.e. where it occurred. Then track.
            if result.is_none() && conflict_region.is_none() {
                // info!("TRACE: {:?}", region);
                conflict_region.replace((*region).clone());
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