use std::cmp::Ordering;
use std::fmt::Debug;
use std::ops::Index;
use chrono::{DateTime, TimeDelta, Utc};
use tai_time::MonotonicTime;
use crate::interpreter::history::History;
use crate::interpreter::regions::Region;
use crate::observations::Observation;

pub(crate) trait Prune {
    fn prune(&self, now: Self) -> bool;
}
impl Prune for DateTime<Utc> {
    fn prune(&self, now: Self) -> bool {
        (now - self) > TimeDelta::seconds(30)
    }
}

impl Prune for MonotonicTime {
    fn prune(&self, now: Self) -> bool {
        now.duration_since(self.clone()).as_secs() > 30
    }
}


impl<T: PartialOrd + Clone + Prune + Debug> History<T> {
    // Insertion Automata
    pub fn insert(&mut self, observation: Observation<T>, now: T) -> Vec<Region<T>> {
        // info!("History Before: {:?}", self.list);

        let mut cursor = self.list.cursor_front_mut();
        let mut pruned = Vec::new();
        loop {
            match cursor.current() {
                Some(R_i) => match R_i.compare_with_observation(&observation) {
                    None => if cursor.peek_next().is_some_and( // If the next region exists and...
                        |next_region| next_region
                            .compare_with_observation(&observation)
                            .is_none() // Is unorderable against the current observation.
                    ){
                        // Then enter Merge-Mode!
                        let mut merge_into = cursor.remove_current().unwrap(); // Save current region. (and moves to R_{i+1})
                        for element in cursor.remove_current().unwrap().observations { // Remove R_{i+1} (moves cursor to i+2)
                            merge_into.insert(element) // Take all of R_i+1, we already know it's unorderable.
                        }
                        // Greedily consume until end or less!
                        loop {
                            match cursor.current() {
                                Some(region) => match region.compare_with_observation(&observation) {
                                    // If less than region, finish capture!
                                    Some(Ordering::Less) => break,
                                    Some(Ordering::Equal) => unreachable!(),
                                    _ => {
                                        // Otherwise, keep capturing!
                                        for element in cursor.remove_current().unwrap().observations {
                                            merge_into.insert(element);
                                        }
                                    }
                                }
                                // If at end, finish capture!
                                None => break
                            }
                        }
                        // Once capture complete- add observation.
                        merge_into.insert(observation);
                        // info!("INCOMP WITH ONE, and CAPTURED MANY - INSERT MERGED");

                        cursor.insert_before(merge_into);
                        // info!("History After: {:?}", &self.list);
                        return pruned // Ending State!
                    } else {
                        // info!("INCOMP WITH ONE, and LESS THAN NEXT - INSERT HERE");
                        // Otherwise, insert into current!
                        cursor.current().unwrap().insert(observation);
                        // info!("History After: {:?}", &self.list);
                        return pruned; // Ending state!
                    },
                    // If less than all, insert new region before (implicitly got here by being > previous)
                    Some(Ordering::Less) => {
                        // info!("LESS - INSERT BEFORE");

                        cursor.insert_before(Region::new(observation));
                        // info!("History After: {:?}", &self.list);
                        return pruned; // Ending state!
                    },
                    // If greater than all, keep going.
                    Some(Ordering::Greater) => {
                        // info!("GREATER - CONTINUE");

                        if cursor.current().unwrap().observations.index(0).interval.0.prune(now.clone()) {
                            // info!("Pruning: {:?}", cursor.current());
                            pruned.push(cursor.remove_current().unwrap());
                        } else {
                            cursor.move_next()
                        }
                    },
                    // Observations are unique!
                    Some(Ordering::Equal) => unreachable!()
                },
                None => {
                    // info!("REACHED END- INSERT BEFORE");
                    // Insert new region with O at i.
                    cursor.insert_before(Region::new(observation));
                    // info!("History After: {:?}", &self.list);
                    return pruned; // Ending State!
                }
            }
        }
    }
}