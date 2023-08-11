mod test;

use petgraph::graph::EdgeReference;
use petgraph::visit::EdgeRef;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::Hash;

pub type Weight = u64;
pub type NodeId = usize;
pub type UntranslatedNodeId = String;

#[derive(Debug, Clone)]
pub struct Weights(pub Vec<Weight>);

#[derive(Debug, Clone)]
pub struct WeightsTuple {
    pub weights: Vec<Weight>,
    pub hidden_weights: Option<Vec<Weight>>,
}

#[derive(Debug, Clone, Hash)]
pub struct Label<T> {
    pub values: Vec<u64>,
    pub hidden_values: Option<Vec<u64>>,
    pub path: Vec<T>,
    pub node_id: T,
}

impl Label<NodeId> {
    pub fn new_along(&self, edge: &EdgeReference<WeightsTuple>) -> Label<NodeId> {
        let weight = edge.weight();
        let values = self
            .values
            .iter()
            .zip(weight.weights.iter())
            .map(|(a, b)| a + b)
            .collect();
        let hidden_values = match &self.hidden_values {
            Some(hidden_values) => {
                if let Some(hidden_weights) = &weight.hidden_weights {
                    Some(
                        hidden_values
                            .iter()
                            .zip(hidden_weights.iter())
                            .map(|(a, b)| a + b)
                            .collect(),
                    )
                } else {
                    None
                }
            }
            None => None,
        };

        let mut path = self.path.clone();
        let target_node_id = edge.target().index();
        path.push(target_node_id);
        Label {
            values,
            path,
            node_id: target_node_id,
            hidden_values,
        }
    }

    // returns true if the label weakly dominates the other label
    // this is the case if it either strictly dominates the other label
    // or if it is equal to the other label
    fn weakly_dominates(&self, other: &Label<NodeId>) -> bool {
        self.values
            .iter()
            .zip(other.values.iter())
            .all(|(a, b)| a <= b)
    }

    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        for node in &self.path {
            if visited.contains(node) {
                return true;
            }
            visited.insert(node);
        }
        false
    }
}

impl Ord for Label<NodeId> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // lexicographical order, but the smaller the better
        // we need a "min-heap" as queue
        match self.values.cmp(&other.values) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => Ordering::Equal,
        }
    }
}

impl<T> PartialOrd for Label<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // lexicographical order, but the smaller the better
        // we need a "min-heap" as queue
        match self.values.cmp(&other.values) {
            Ordering::Less => Some(Ordering::Greater),
            Ordering::Greater => Some(Ordering::Less),
            Ordering::Equal => Some(Ordering::Equal),
        }
    }
}

impl<T> PartialEq for Label<T> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}

impl Eq for Label<NodeId> {}
impl Eq for Label<String> {}

#[derive(Debug)]
pub struct Bag<T: Eq + Hash> {
    pub labels: HashSet<Label<T>>,
}

impl Bag<NodeId> {
    pub fn new_start_bag(start_label: Label<NodeId>) -> Bag<NodeId> {
        let mut labels = HashSet::new();
        labels.insert(start_label);
        Bag { labels }
    }

    pub fn new_empty() -> Bag<NodeId> {
        Bag {
            labels: HashSet::new(),
        }
    }

    pub fn add_if_necessary(&mut self, label: Label<NodeId>) -> bool {
        if self.content_dominates(&label) {
            return false;
        }
        self.remove_dominated_by(&label);
        self.labels.insert(label);
        true
    }

    pub fn content_dominates(&self, label: &Label<NodeId>) -> bool {
        for l in &self.labels {
            if l.weakly_dominates(label) {
                return true;
            }
        }
        false
    }

    fn remove_dominated_by(&mut self, label: &Label<NodeId>) {
        self.labels.retain(|l| !label.weakly_dominates(l));
    }
}
