mod test;

use petgraph::graph::EdgeReference;
use petgraph::visit::EdgeRef;
use std::collections::HashSet;
use std::hash::Hash;

pub type Weight = u64;
pub type NodeId = usize;

#[derive(Debug, Clone)]
pub struct Weights(pub Vec<Weight>);

#[derive(Debug, Clone, Hash)]
pub struct Label {
    pub values: Vec<Weight>,
    pub path: Vec<NodeId>,
    pub current: NodeId,
}

impl Label {
    pub fn new_along(&self, edge: &EdgeReference<Weights>) -> Label {
        let values = self
            .values
            .iter()
            .zip(edge.weight().0.iter())
            .map(|(a, b)| a + b)
            .collect();

        let mut path = self.path.clone();
        let current = edge.target().index();
        path.push(current);
        Label {
            values,
            path,
            current,
        }
    }

    // returns true if the label weakly dominates the other label
    // this is the case if it either strictly dominates the other label
    // or if it is equal to the other label
    fn weakly_dominates(&self, other: &Label) -> bool {
        self.values
            .iter()
            .zip(other.values.iter())
            .all(|(a, b)| a >= b)
    }
}

impl Ord for Label {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.values.cmp(&other.values) // lexicographical order
    }
}

impl PartialOrd for Label {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.values.cmp(&other.values)) // lexicographical order
    }
}

impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl Eq for Label {}

#[derive(Debug, PartialEq)]
pub struct Bag {
    pub labels: HashSet<Label>,
}

impl Bag {
    pub fn new_start_bag(start_label: Label) -> Bag {
        let mut labels = HashSet::new();
        labels.insert(start_label);
        Bag { labels }
    }

    pub fn new_empty() -> Bag {
        Bag {
            labels: HashSet::new(),
        }
    }

    pub fn add_if_necessary(&mut self, label: Label) -> bool {
        if self.content_dominates(&label) {
            return false;
        }
        self.remove_dominated_by(&label);
        self.labels.insert(label);
        true
    }

    pub fn content_dominates(&self, label: &Label) -> bool {
        for l in &self.labels {
            if l.weakly_dominates(label) {
                return true;
            }
        }
        false
    }

    fn remove_dominated_by(&mut self, label: &Label) {
        self.labels.retain(|l| !label.weakly_dominates(l));
    }
}
