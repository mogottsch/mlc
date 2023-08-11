use crate::bag::*;
use bimap::BiMap;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{Graph, Undirected};
use std::collections::{BinaryHeap, HashMap};
use std::error::Error;
use std::fmt::Display;
use std::fs::{read_to_string, File};
use std::hash::Hash;
use std::io::Write;
use std::num::ParseIntError;
use std::str::FromStr;

mod test;

pub struct MLC {
    graph: Graph<(), WeightsTuple, Undirected>,
    label_length: usize,
    hidden_label_length: usize,
    node_map: BiMap<String, usize>,
    update_label_func: Option<fn(&Label<usize>, &Label<usize>) -> Label<usize>>,
}

pub type Bags<T> = HashMap<T, Bag<T>>;

impl MLC {
    pub fn new(
        g: Graph<(), WeightsTuple, Undirected>,
        node_map: BiMap<String, usize>,
        update_label_func: Option<fn(&Label<usize>, &Label<usize>) -> Label<usize>>,
    ) -> Result<MLC, Box<dyn Error>> {
        if g.edge_count() == 0 {
            return Err("Graph has no edges".into());
        }

        let mut n_labels = usize::MAX;
        let mut n_hidden_labels = usize::MAX;

        for node in g.node_indices() {
            for edge in g.edges(node) {
                if n_labels == usize::MAX {
                    n_labels = edge.weight().weights.len();
                    continue;
                }
                if n_labels != edge.weight().weights.len() {
                    return Err("Graph has inconsistent edge weights".into());
                }

                if let Some(hidden_weights) = edge.weight().hidden_weights.as_ref() {
                    if n_hidden_labels == usize::MAX {
                        n_hidden_labels = hidden_weights.len();
                        continue;
                    }
                    if n_hidden_labels != hidden_weights.len() {
                        return Err("Graph has inconsistent hidden edge weights".into());
                    }
                }
            }
        }

        Ok(MLC {
            graph: g,
            label_length: n_labels,
            node_map,
            hidden_label_length: n_hidden_labels,
            update_label_func,
        })
    }

    pub fn run(&self, start: String) -> Bags<String> {
        if let Some(start) = self.node_map.get_by_left(&start) {
            return self.run_resetted(*start);
        }
        panic!("Start node not found");
    }

    fn run_resetted(&self, start: usize) -> Bags<String> {
        let mut queue: BinaryHeap<Label<usize>> = BinaryHeap::new();
        let mut bags: Bags<usize> = HashMap::new();
        let hidden_values = if self.hidden_label_length != usize::MAX {
            Some(vec![0; self.hidden_label_length])
        } else {
            None
        };
        let start_label = Label {
            values: vec![0; self.label_length],
            hidden_values,
            path: vec![start],
            node_id: start,
        };
        queue.push(start_label.clone());
        bags.insert(start, Bag::new_start_bag(start_label));

        let mut counter = 0;

        while let Some(label) = queue.pop() {
            let node_id = label.node_id;

            for edge in self.graph.edges(NodeIndex::new(node_id)) {
                let old_label = label.clone();
                let mut new_label = label.new_along(&edge);
                if let Some(update_label_func) = self.update_label_func {
                    new_label = update_label_func(&old_label, &new_label);
                }
                let target_bag = bags
                    .entry(edge.target().index())
                    .or_insert_with(Bag::new_empty);
                if target_bag.add_if_necessary(new_label.clone()) {
                    queue.push(new_label);
                }
            }

            counter += 1;
            if counter % 1000 == 0 {
                println!("queue size: {}", queue.len());
                self.write_bags_as_csv(&self.translate_bags(&bags), "labels.csv");
            }
        }

        self.translate_bags(&bags)
    }

    fn translate_bags(&self, bags: &Bags<usize>) -> Bags<String> {
        let mut translated_bags: Bags<String> = HashMap::new();
        for (node_id, bag) in bags {
            let translated_node_id = self.node_map.get_by_right(node_id).unwrap();
            let translated_bag = Bag {
                labels: bag
                    .labels
                    .iter()
                    .map(|label| Label {
                        node_id: translated_node_id.clone(),
                        path: label
                            .path
                            .iter()
                            .map(|n| self.node_map.get_by_right(n).unwrap().to_string())
                            .collect(),
                        values: label.values.clone(),
                        hidden_values: label.hidden_values.clone(),
                    })
                    .collect(),
            };
            translated_bags.insert(translated_node_id.to_string(), translated_bag);
        }
        translated_bags
    }

    #[allow(dead_code)]
    fn write_node_map_as_csv(&self, filename: &str) {
        let mut file = File::create(filename).unwrap();
        writeln!(file, "node_id,mlc_node_id").unwrap();
        for (key, value) in &self.node_map {
            writeln!(file, "{},{}", key, value).unwrap();
        }
    }

    #[allow(dead_code)]
    fn write_bags_as_csv(&self, bags: &Bags<String>, filename: &str) {
        let mut file = File::create(filename).unwrap();
        writeln!(file, "node_id|path|values").unwrap();
        for (node_id, bag) in bags {
            for label in &bag.labels {
                writeln!(
                    file,
                    "{}|{}|{}",
                    node_id,
                    label.path.join(","),
                    label
                        .values
                        .iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<String>>()
                        .join(",")
                )
                .unwrap();
            }
        }
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

pub fn read_bags(path: &str) -> Result<Bags<usize>, Box<dyn Error>> {
    let mut bags: Bags<usize> = HashMap::new();
    for line in read_to_string(path)?.lines().skip(1) {
        let label_entry: LabelEntry = line.parse()?;
        let label = Label {
            values: label_entry.values.clone(),
            hidden_values: None,
            path: label_entry.path.clone(),
            node_id: label_entry.node_id,
        };
        let bag = bags
            .entry(label_entry.node_id)
            .or_insert_with(Bag::new_empty);
        bag.add_if_necessary(label);
    }
    Ok(bags)
}

pub fn write_bags<T: Eq + Hash + Display>(
    bags: &Bags<T>,
    path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut file = std::fs::File::create(path)?;
    let header = "node_id|path|weights\n";
    file.write_all(header.as_bytes())?;

    for bag in bags.values() {
        for label in bag.labels.iter() {
            let mut values = label.values.clone();
            if let Some(hidden_values) = &label.hidden_values {
                values.extend(hidden_values);
            }
            let line = format!(
                "{}|{}|{}\n",
                label.node_id,
                label
                    .path
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
                values
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            );

            file.write_all(line.as_bytes())?;
        }
    }
    Ok(())
}
