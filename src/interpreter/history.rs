use std::collections::LinkedList;
use crate::interpreter::regions::Region;
use crate::value::Value;

pub struct History<T>(pub(crate) LinkedList<Region<T>>);

impl<T> History<T> {
    pub fn new() -> Self {
        Self(LinkedList::new())
    }

    pub fn apply(&mut self, mut value: Option<Value>) -> Option<Value> {
        for mut region in self.0 {
            value = region.apply(value);
        }
        value
    }
}