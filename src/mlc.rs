use crate::bag::*;
use bimap::BiMap;
use log::{debug, info, warn};
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::{Directed, Graph};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::fs::{read_to_string, File};
use std::hash::Hash;
use std::io::Write;
use std::num::ParseIntError;
use std::str::FromStr;
use std::time::Instant;

use self::limit::Limits;

mod limit;
mod test;

type UpdateLabelFunc = fn(&Label<usize>, &Label<usize>) -> Label<usize>;

pub struct MLC<'a, T: std::cmp::Eq + std::hash::Hash + std::marker::Copy> {
    // problem state
    graph: &'a Graph<Vec<T>, WeightsTuple, Directed>,
    update_label_func: Option<UpdateLabelFunc>,

    // config
    node_map: Option<BiMap<String, usize>>,
    debug: bool,
    disable_paths: bool,
    enable_limit: bool,

    // helper variables
    weight_length: usize,
    hidden_weights_length: usize,

    // internal state
    bags: Bags<usize>,
    queue: BinaryHeap<Label<usize>>,
    limits: Limits<T>,
}

pub type Bags<T> = HashMap<T, Bag<T>>;

#[derive(Debug)]
pub enum MLCError {
    StartNodeNotFound(String),
    NodeMapNotSet,
    UnknownNodeId(usize),
    EmptyStartingQueue,
}

impl fmt::Display for MLCError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MLCError::StartNodeNotFound(start_node) => {
                write!(f, "Start node not found: {}", start_node)
            }
            MLCError::NodeMapNotSet => write!(f, "Node map not set"),
            MLCError::UnknownNodeId(node_id) => write!(f, "Unknown node id: {}", node_id),
            MLCError::EmptyStartingQueue => write!(
                f,
                "Starting queue is empty. Specify either a start node or a starting queue."
            ),
        }
    }
}

impl Error for MLCError {}

impl<T: std::cmp::Eq + std::hash::Hash + std::marker::Copy> MLC<'_, T> {
    pub fn new(g: &Graph<Vec<T>, WeightsTuple, Directed>) -> Result<MLC<T>, Box<dyn Error>> {
        if g.edge_count() == 0 {
            return Err("Graph has no edges".into());
        }

        let sample_edge_weight = g.edge_references().next().unwrap().weight();
        let n_weights = sample_edge_weight.weights.len();
        let n_hidden_weights = sample_edge_weight.hidden_weights.len();

        for node in g.node_indices() {
            for edge in g.edges(node) {
                if n_weights != edge.weight().weights.len() {
                    return Err("Graph has inconsistent edge weights".into());
                }
                if n_hidden_weights != edge.weight().hidden_weights.len() {
                    return Err("Graph has inconsistent hidden edge weights".into());
                }
            }
        }

        let mut limits = Limits::new();
        let categories = g
            .node_indices()
            .map(|node| g.node_weight(node).unwrap())
            .flatten()
            .collect::<HashSet<_>>();
        for category in categories {
            limits.add_category(category.clone());
        }

        Ok(MLC {
            graph: g,
            bags: HashMap::new(),
            queue: BinaryHeap::new(),
            weight_length: n_weights,
            node_map: None,
            disable_paths: false,
            hidden_weights_length: n_hidden_weights,
            update_label_func: None,
            debug: false,
            limits,
            enable_limit: false,
        })
    }

    pub fn set_update_label_func(
        &mut self,
        update_label_func: fn(&Label<usize>, &Label<usize>) -> Label<usize>,
    ) {
        self.update_label_func = Some(update_label_func);
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn set_node_map(&mut self, node_map: BiMap<String, usize>) {
        self.node_map = Some(node_map);
    }

    pub fn set_disable_paths(&mut self, disable_paths: bool) {
        self.disable_paths = disable_paths;
    }

    pub fn set_enable_limit(&mut self, enable_limit: bool) {
        self.enable_limit = enable_limit;
    }

    /// Sets the starting bags and derives the starting queue from them.
    ///
    /// The bags should be in a consistent state, meaning that the labels in the bags should
    /// not dominate each other.
    ///
    /// # Arguments
    ///
    /// * `bags` - A HashMap of bags, where the key is the node id and the value is the bag.
    pub fn set_bags(&mut self, bags: Bags<usize>) {
        assert!(!bags.is_empty());
        self.bags = bags;
        let mut label_node_tuples = vec![];
        for bag in self.bags.values() {
            for label in &bag.labels {
                let node_weight = self
                    .graph
                    .node_weight(NodeIndex::new(label.node_id))
                    .unwrap();
                if self.enable_limit && node_weight.len() > 0 {
                    label_node_tuples.push((label.clone(), node_weight));
                }

                assert_eq!(label.values.len(), self.weight_length);
                assert_eq!(
                    label.hidden_values.len(),
                    self.hidden_weights_length,
                    "new label length: {} != {}: old label length",
                    label.hidden_values.len(),
                    self.hidden_weights_length
                );

                self.queue.push(label.clone());
            }
        }
        for (label, node_weight) in label_node_tuples {
            self.update_limits(&label, node_weight);
        }
    }

    pub fn set_start_node(&mut self, start_node: usize) {
        let hidden_values = vec![0; self.hidden_weights_length];

        let start_path = if self.disable_paths {
            vec![]
        } else {
            vec![start_node]
        };
        let start_label = Label {
            values: vec![0; self.weight_length],
            hidden_values,
            path: start_path,
            node_id: start_node,
        };
        self.queue.push(start_label.clone());
        self.bags
            .insert(start_node, Bag::new_start_bag(start_label));
    }

    pub fn set_start_node_with_time(&mut self, start_node: usize, time: usize) {
        let hidden_values = vec![0; self.hidden_weights_length];
        let mut values = vec![0; self.weight_length];
        values[0] = time.try_into().unwrap();

        let start_path = if self.disable_paths {
            vec![]
        } else {
            vec![start_node]
        };

        let start_label = Label {
            values,
            hidden_values,
            path: start_path,
            node_id: start_node,
        };
        self.queue.push(start_label.clone());
        self.bags
            .insert(start_node, Bag::new_start_bag(start_label));
    }

    pub fn set_external_start_node(&mut self, start_node: String) -> Result<(), MLCError> {
        let start_node = self
            .node_map
            .as_ref()
            .ok_or(MLCError::NodeMapNotSet)?
            .get_by_left(&start_node)
            .ok_or(MLCError::StartNodeNotFound(start_node))?;
        self.set_start_node(*start_node);
        Ok(())
    }

    /// Run the MLC algorithm on the graph, starting at the given node.
    /// The node id is expected to be the integer node id.
    ///
    /// # Arguments
    /// * `start` - The node to start the algorithm at.
    ///
    /// # Returns
    /// * `Bags<usize>` - The bags of each node.
    pub fn run(&mut self) -> Result<&Bags<usize>, MLCError> {
        debug!("mlc config: {:?}", self);

        let mut counter = 0;
        let mut time = Instant::now();

        let mut n_limit_exceeded = 0;

        if self.enable_limit && !self.limits.is_initialized() {
            panic!("Limits must be initialized before running the algorithm");
        }

        while let Some(label) = self.queue.pop() {
            if self.enable_limit && self.exceeds_limit(&label) {
                n_limit_exceeded += 1;
                continue;
            }

            let node_id = label.node_id;

            // check if this label is still in the bag of its node, if not, we can skip it
            // to speed up the algorithm (~20%)
            if !self
                .bags
                .get(&node_id)
                .ok_or(MLCError::UnknownNodeId(node_id))?
                .labels
                .contains(&label)
            {
                continue;
            }

            for edge in self.graph.edges(NodeIndex::new(node_id)) {
                let old_label = label.clone();
                let mut new_label = label.new_along(&edge, self.disable_paths);
                if let Some(update_label_func) = self.update_label_func {
                    new_label = update_label_func(&old_label, &new_label);
                }
                let target_bag = self
                    .bags
                    .entry(edge.target().index())
                    .or_insert_with(Bag::new_empty);
                if target_bag.add_if_necessary(new_label.clone()) {
                    let target_node_values = self
                        .graph
                        .node_weight(edge.target())
                        .ok_or(MLCError::UnknownNodeId(edge.target().index()))?;
                    if self.enable_limit && target_node_values.len() > 0 {
                        self.update_limits(&new_label, target_node_values);
                    }
                    self.queue.push(new_label);
                }
            }

            counter += 1;
            // print queue size every 1000 iterations
            // if debug is enabled, write labels to csv every 10 seconds
            if counter % 1000 == 0 {
                debug!("queue size: {}", self.queue.len());
                if self.debug {
                    let duration = time.elapsed();
                    if duration.as_secs() > 10 {
                        info!("writing labels to csv");
                        write_bags(&self.translate_bags(&self.bags), "data/labels.csv").unwrap();
                        time = Instant::now();
                    }
                }
            }
        }

        if self.enable_limit && n_limit_exceeded > 0 {
            debug!(
                "{} labels were discarded because they exceeded the limit",
                n_limit_exceeded
            );
        }

        Ok(&self.bags)
    }

    fn translate_bags(&self, bags: &Bags<usize>) -> Bags<String> {
        let node_map = self
            .node_map
            .as_ref()
            .expect("node_map must be passed when calling translate_bags");
        let mut translated_bags: Bags<String> = HashMap::new();
        for (node_id, bag) in bags {
            let translated_node_id = node_map.get_by_right(node_id).unwrap();
            let translated_bag = Bag {
                labels: bag
                    .labels
                    .iter()
                    .map(|label| Label {
                        node_id: translated_node_id.clone(),
                        path: label
                            .path
                            .iter()
                            .map(|n| node_map.get_by_right(n).unwrap().to_string())
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
        let node_map = self
            .node_map
            .as_ref()
            .expect("node_map must be passed when calling write_node_map_as_csv");

        let mut file = File::create(filename).unwrap();
        writeln!(file, "node_id,mlc_node_id").unwrap();
        for (key, value) in node_map {
            writeln!(file, "{},{}", key, value).unwrap();
        }
    }

    fn exceeds_limit(&mut self, label: &Label<usize>) -> bool {
        let values = &label.values;
        if values.len() != 2 {
            return false;
        }
        let cost = values[1];
        let time = values[0];
        let result = self.limits.is_limit_exceeded(cost, time);
        return result;
    }

    fn update_limits(&mut self, label: &Label<usize>, node_values: &Vec<T>) {
        for value in node_values.iter() {
            let category = value;
            let cost = label.values[1];
            let time = label.values[0];
            self.limits.update_limit(*category, cost, time);
        }
    }
}

impl<T: std::cmp::Eq + std::hash::Hash + std::marker::Copy> fmt::Debug for MLC<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MLC")
            .field("debug", &self.debug)
            .field("disable_paths", &self.disable_paths)
            .field("enable_limit", &self.enable_limit)
            .field(
                "update_label_func_defined",
                &self.update_label_func.is_some(),
            )
            .field("weight_length", &self.weight_length)
            .field("hidden_weights_length", &self.hidden_weights_length)
            .finish()
    }
}
// impl<T> fmt::Debug for MLC<T> {}

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
            hidden_values: vec![],
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
            values.extend(label.hidden_values.clone());
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
