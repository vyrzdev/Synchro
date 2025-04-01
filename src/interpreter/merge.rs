use crate::observations::Observation;
use crate::predicates::DefinitionPredicate;
use crate::predicates::DefinitionPredicate::Unknown;

fn merge_procedure<T>(observations: &Vec<Observation<T>>) -> DefinitionPredicate {
    let mut all_mutations = true;
    let mut sum = 0;

    for observation in observations {
        match observation.definition_predicate {
            DefinitionPredicate::AllMut(delta) if all_mutations => {
                sum += delta
            },
            DefinitionPredicate::AllMut(_) if !all_mutations => return Unknown, // Mutations only commute with mutations.
            DefinitionPredicate::LastAssn(_) => {
                all_mutations = false;
            }
            _ => return Unknown // Unknowns and Transitions cannot be merged.
        }
    }

    if all_mutations {
        return DefinitionPredicate::AllMut(sum);
    } else {
        return Unknown // TODO: LastAssn Merge.
    }
}
