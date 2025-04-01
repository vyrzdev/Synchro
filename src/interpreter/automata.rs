#![feature(linked_list_cursors)]
use std::cmp::Ordering;
use std::thread::current;
use crate::interpreter::history::History;
use crate::interpreter::regions::Region;
use crate::observations::Observation;


impl<T> History<T> {
    // Insertion Automata
    pub fn insert(&mut self, observation: Observation<T>) {
        let mut cursor = self.0.cursor_front_mut();
        loop {
            match cursor.current() {
                Some(R_i) => match R_i.compare_with_observation(&observation) {
                    None => if cursor.peek_next().is_some_and( // If the next region exists and...
                        |next_region| next_region
                            .compare_with_observation(&observation)
                            .is_none() // Is unorderable against the current observation.
                    ){
                        // Then enter Merge-Mode!
                        let merge_into = cursor.current().unwrap(); // Save ref to current.
                        cursor.move_next(); // Move to R_i+1
                        for element in cursor.remove_current().unwrap() { // Remove R_{i+1} (moves cursor to i+2)
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
                                        for element in cursor.remove_current().unwrap() {
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
                        return // Ending State!
                    } else {
                        // Otherwise, insert into current!
                        cursor.current().unwrap().insert(observation);
                        return; // Ending state!
                    },
                    // If less than all, insert new region before (implicitly got here by being > previous)
                    Some(Ordering::Less) => {
                        cursor.insert_before(Region::new(observation));
                        return; // Ending state!
                    },
                    // If greater than all, keep going.
                    Some(Ordering::Greater) => cursor.move_next(),
                    // Observations are unique!
                    Some(Ordering::Equal) => unreachable!()
                },
                None => {
                    // Insert new region with O at i.
                    cursor.insert_before(Region::new(observation));
                    return; // Ending State!
                }
            }
        }
    }
}