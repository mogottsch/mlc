use crate::bag::*;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Graph;
use std::collections::{BinaryHeap, HashMap};
use std::error::Error;
use std::fs::read_to_string;
use std::num::ParseIntError;
use std::str::FromStr;

mod test;

pub struct MLC {
    graph: Graph<u64, Weights>,
    label_length: usize,
}

pub type Bags = HashMap<usize, Bag>;

impl MLC {
    pub fn new(g: Graph<u64, Weights>) -> Result<MLC, Box<dyn Error>> {
        if g.edge_count() == 0 {
            return Err("Graph has no edges".into());
        }

        let mut n_labels = usize::MAX;

        for node in g.node_indices() {
            for edge in g.edges(node) {
                if n_labels == usize::MAX {
                    n_labels = edge.weight().0.len();
                    continue;
                }
                if n_labels != edge.weight().0.len() {
                    return Err("Graph has inconsistent edge weights".into());
                }
            }
        }

        Ok(MLC {
            graph: g,
            label_length: n_labels,
        })
    }
    pub fn run(self, start: usize) -> Bags {
        let mut queue: BinaryHeap<Label> = BinaryHeap::new();
        let mut bags: Bags = HashMap::new();
        let start_label = Label {
            values: vec![0; self.label_length],
            path: vec![start],
            current: start,
        };
        queue.push(start_label.clone());
        bags.insert(start, Bag::new_start_bag(start_label));

        while let Some(label) = queue.pop() {
            let current = label.current;

            for edge in self.graph.edges(NodeIndex::new(current)) {
                let new_label = label.new_along(&edge);
                let target_bag = bags
                    .entry(edge.target().index())
                    .or_insert_with(Bag::new_empty);
                if target_bag.add_if_necessary(new_label.clone()) {
                    queue.push(new_label);
                }
            }
        }

        bags
    }
}

#[derive(Debug)]
struct LabelEntry {
    node_id: NodeId,
    path: Vec<NodeId>,
    values: Vec<Weight>,
}

impl FromStr for LabelEntry {
    type Err = ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // node_id|path_node1,path_node2,...|value1,value2,...
        let mut parts = s.split('|');

        let node_id = parts.next().unwrap().parse::<NodeId>()?;
        let path = parts
            .next()
            .unwrap()
            .split(',')
            .map(|s| s.parse::<NodeId>())
            .collect::<Result<Vec<NodeId>, _>>()?;
        let values = parts
            .next()
            .unwrap()
            .split(',')
            .map(|s| s.parse::<Weight>())
            .collect::<Result<Vec<Weight>, _>>()?;
        Ok(LabelEntry {
            node_id,
            path,
            values,
        })
    }
}

pub fn read_bags(path: &str) -> Result<Bags, Box<dyn Error>> {
    let mut bags: Bags = HashMap::new();
    for line in read_to_string(path)?.lines() {
        let label_entry: LabelEntry = line.parse()?;
        let label = Label {
            values: label_entry.values.clone(),
            path: label_entry.path.clone(),
            current: label_entry.node_id,
        };
        let bag = bags
            .entry(label_entry.node_id)
            .or_insert_with(Bag::new_empty);
        bag.add_if_necessary(label);
    }
    Ok(bags)
}
