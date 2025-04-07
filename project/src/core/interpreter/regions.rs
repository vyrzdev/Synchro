use std::cmp::Ordering;
use crate::core::interpreter::merge::merge_procedure;
use crate::core::observations::Observation;
use crate::core::predicates::DefinitionPredicate;
use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Region<T: PartialOrd + Clone> {
    pub(crate) observations: Vec<Observation<T>>,
    pub(crate) cached_definition: Option<DefinitionPredicate>
}

impl<T: PartialOrd + Clone> PartialEq for Region<T> {
    fn eq(&self, _: &Self) -> bool {
        false // Regions are unique!
    }
}

impl<T: PartialOrd + Clone> Region<T> {
    pub(crate) fn new(obs: Observation<T>) -> Region<T> {
        Region {
            observations: vec![obs.clone()],
            cached_definition: Some(obs.definition_predicate)
        }
    }

    pub(crate) fn insert(&mut self, observation: Observation<T>) {
        // TODO: Topological Insert to optimise maximal elements.
        self.observations.push(observation);
        // Reset definition, as has changed.
        self.cached_definition = None;
    }

    pub(crate) fn apply(&mut self, value: Option<Value>) -> Option<Value> {
        match self.cached_definition.as_ref() {
            // If some definition cached, apply it.
            Some(definition) => definition.apply(value),
            None => {
                // If no definition cached, must have had insert- n > 1.
                // Therefore, apply merge procedure.
                // If un-mergeable, returns Unknown which applied gives None
                let merged = merge_procedure(&self.observations); // TODO: Wire Up Merge Procedure.
                self.cached_definition = Some(merged.clone());
                merged.apply(value)
            }
        }
    }

    pub(crate) fn compare_with_observation(&self, obs: &Observation<T>) -> Option<Ordering> {
        let mut less_than_comparable = true; // True until proven otherwise
        let mut greater_than_comparable = true; // ditto.
        let mut incomparable_with_some = false; // False until seen.

        for contained in &self.observations {
            match obs.partial_cmp(contained) {
                // Not greater than all, as less than one.
                Some(Ordering::Less) => {greater_than_comparable = false},
                // Not less than all, as greater than one.
                Some(Ordering::Greater) => {less_than_comparable = false},
                // There exists some O_R in R where O ~ O_R
                None => {incomparable_with_some = true},
                // Observations are unique!
                _ => unreachable!()
            }
        }

        if less_than_comparable && !incomparable_with_some {
            Some(Ordering::Less) // Less than ALL in region.
        } else if greater_than_comparable && !incomparable_with_some {
            Some(Ordering::Greater) // Greater than ALL in region.
        } else {
            None // Incomparable with some in region.
        }
    }
}