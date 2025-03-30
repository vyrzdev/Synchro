#![feature(linked_list_cursors)]
use std::cmp::Ordering;
use std::collections::LinkedList;
use std::fmt::{Display, Formatter, Pointer};
use crate::observations::Observation;

#[derive(Debug, Clone)]
pub struct Node {
    pub observation: Observation,
    pub n_incomparables: u64,
    pub name: u64
}

impl Node {
    pub(crate) fn new(obs: Observation, name: u64) -> Node {
        Node {
            observation: obs,
            n_incomparables: 0,
            name,
        }
    }
}

impl PartialEq for Node{
    fn eq(&self, other: &Self) -> bool {
        return false; // Nodes are UNIQUE!
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.observation.partial_cmp(&other.observation) {
            Some(Ordering::Equal) => {
                panic!("Should not be possible for observations to be equal!");
            },
            Some(ordering) => Some(ordering),
            None => {
                return None
            }
        }

    }
}

#[derive(Debug)]
pub struct History {
    nodes: LinkedList<Node>
}

impl Display for History {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.nodes.iter().map(|x| x.name.to_string().clone()).collect::<Vec<String>>().join(", "))
    }
}

impl History {
    pub fn new() -> History {
        History {
            nodes: LinkedList::new()
        }
    }

    // Do Iterator Later.

    pub fn add_node(&mut self, mut new_node: Node) {
        // Interpreter Automaton; Ref XXX
        if self.nodes.len() == 0 {
            // Initialise History
            self.nodes.push_back(new_node);
        } else {
            let mut cursor = self.nodes.cursor_front_mut();

            let mut inserted = false;
            loop {
                match cursor.current() {
                    Some(node) => match new_node.partial_cmp(node) {
                        None => {
                            // N- Incomparables, REF: XXX
                            node.n_incomparables += 1;
                            new_node.n_incomparables += 1;
                        },
                        Some(ordering) => match ordering {
                            Ordering::Less if !inserted => {
                                // If less, insert then and there.
                                cursor.insert_before(new_node.clone());
                                inserted = true;
                            },
                            Ordering::Equal => panic!("Nodes are UNIQUE!"),
                            _ => {
                                // Otherwise, wait to insert till end.
                            }
                        }
                    },
                    None if !inserted => {
                        // Reached End and not yet inserted!
                        cursor.push_back(new_node.clone()); // Add node!
                        break; // end iteration.
                    },
                    _ => {
                        // Reached end and inserted.
                        break;
                    }
                }
                cursor.move_next();
            }
        }
    }
}

pub fn traverse_history(history: &mut History) {
    let mut capture: Vec<Node> = Vec::with_capacity(history.nodes.len());

    // Debug Printout:
    for node in history.nodes.iter() {
        print!("{}|", node.observation);
    }
    println!("");

    let mut opened = false;
    for node in history.nodes.iter() {
        match node.n_incomparables {
            0 => {
                if opened {
                    for node in capture.iter() {
                        print!("{} : {}|", node.name, node.observation);
                    }
                    println!("Process: {}", History {
                        nodes: LinkedList::from_iter(capture.clone().into_iter()),
                    });
                    capture.clear();
                    opened = false;
                }

                println!("Process: {}", node.name)
            },
            _ => {
                if !opened {
                    println!("Starting Capture!");
                    opened = true; // Just stops message from printing for every single one!
                }
                capture.push(node.clone())
            }
        }
    }
    println!("Ended with {}", History {
        nodes: LinkedList::from_iter(capture.clone().into_iter()),
    });
}
