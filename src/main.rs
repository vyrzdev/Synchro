extern crate core;

mod value;
mod observations;
mod intervals;
mod interpreter;
mod square;
mod simulation;
mod predicates;
mod ordering;

// Testing Macro to make traces quickly...
macro_rules! make_observation {
    ($start:expr, $end:expr) => {
        Observation {
            interval: (
                MonotonicTime::new($start, 0).unwrap(),
                MonotonicTime::new($end, 0).unwrap(),
            ),
            definition_predicate: DefinitionPredicate::Unknown,
            source: "".to_string(),
            platform_metadata: PlatformMetadata::Polling { poll_count: 0 },
        }
    };
}
// Usage:
// let observations = vec![
//     make_observation!(0, 10),
//     ...
//     make_observation!(25, 65)
// ];


fn main() {

}

