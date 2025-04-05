use crate::predicates::DefinitionPredicate::{AllMut, LastAssn, Transition};
use crate::value::Value;
#[derive(Clone, Debug)]
pub enum DefinitionPredicate {
    Transition(Value, Value), // Transition(s_0, s_1) => K(s_0) = s_1
    AllMut(Value), // all x in K are mut to delta => AllMutation(delta) => K(v) = v + delta
    LastAssn(Value), // last x in K is assn to new => LastAssn(new) => K(v) = new
    Unknown // Unknown => K(v) = undefined.
}
impl DefinitionPredicate {
    pub fn apply(&self, value: Option<Value>) -> Option<Value>{
        match (self, value) {
            // Transitions are defined only for their known input.
            (Transition(s_0, s_1), Some(v)) if &v == s_0 => Some(s_1.clone()),
            // Mutations are defined for any defined input.
            (AllMut(delta), Some(v)) => Some(v + delta),
            // Assignments are defined, even for undefined input.
            (LastAssn(new), _) => Some(new.clone()),
            // Unknown, or any other case is undefined.
            _ => None
        }
    }
}
