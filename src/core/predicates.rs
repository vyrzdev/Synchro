use crate::core::predicates::DefinitionPredicate::{AllMut, LastAssn, Transition};
use crate::value::Value;

/// Definition Predicates.
/// - `Transition(Value, Value)`: Indicates a state transition where `K(s_0) = s_1`.
/// - `AllMut(Value)`: Represents a mutation by a `delta`, defined as `K(v) = v + delta`.
/// - `LastAssn(Value)`: Represents assignment to a new value, defined as `K(v) = new`.
/// - `Unknown`: Represents an undefined transition `K(v) = undefined`
#[derive(Clone, Debug)]
pub enum DefinitionPredicate {
    Transition(Value, Value),
    AllMut(Value),
    LastAssn(Value),
    Unknown
}


impl DefinitionPredicate {
    /// The `apply` method applies the definition predicate to a given option.
    ///
    /// # Parameters
    /// - `value`: An `Option<Value>` that represents the input value to the predicate.
    ///
    /// # Returns
    /// - `Option<Value>`: The transformed value, if applicable, or `None` if the input
    ///   or predicate makes the transformation undefined.
    ///
    /// # Behavior
    /// - For `Transition(s_0, s_1)`, the method returns `s_1` if `value` is `Some(v)` and `v == s_0`.
    ///   Otherwise, it returns `None`.
    /// - For `AllMut(delta)`, the method returns `v + delta` if `value` is `Some(v)`.
    ///   For an undefined `value`, it returns `None`.
    /// - For `LastAssn(new)`, the method returns `new` regardless of whether the input
    ///   is defined or not.
    /// - For `Unknown` or any other cases, the method always returns `None`.
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
