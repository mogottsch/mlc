mod test;

use bimap::BiMap;
use petgraph::Graph;
use petgraph::{graph::NodeIndex, Directed};
use serde::{de, Deserialize, Deserializer};
use std::{error::Error, str::FromStr};

use crate::bag::{NodeId, Weights, WeightsTuple};

#[derive(Debug, serde::Deserialize)]
struct UntranslatedEdge {
    u: String,
    v: String,
    weights: Weights,
    hidden_weights: Option<Weights>,
}

#[derive(Debug, serde::Deserialize)]
struct Edge {
    u: NodeId,
    v: NodeId,
    weights: Weights,
    hidden_weights: Option<Weights>,
}

pub type MLCGraph<T> = Graph<Vec<T>, WeightsTuple, Directed>;

// Reads a graph from a csv file. The csv file should have the following format:
// u,v,weights,hidden_weights
// where u and v are the node names, weights are the weights of the edge, and hidden_weights are the
// hidden weights of the edge. The hidden_weights column is optional.
// The node names can be any string, but they must be unique.
// The weights and hidden_weights columns must be a comma-separated list of integers.
pub fn read_graph_and_reset_ids(
    path: &str,
) -> Result<(MLCGraph<()>, BiMap<String, usize>), Box<dyn Error>> {
    // let mut rdr = csv::Reader::from_path(path)?;
    let mut rdr = csv::ReaderBuilder::new().quote(b'"').from_path(path)?;

    let mut edges = Vec::new();
    for result in rdr.deserialize() {
        let edge: UntranslatedEdge = result?;
        edges.push(edge);
    }

    let mut node_map = BiMap::new();
    let mut node_count = 0;
    for edge in &edges {
        if !node_map.contains_left(&edge.u) {
            node_map.insert(edge.u.clone(), node_count);
            node_count += 1;
        }
        if !node_map.contains_left(&edge.v) {
            node_map.insert(edge.v.clone(), node_count);
            node_count += 1;
        }
    }
    let translated_edges = edges
        .iter()
        .map(|e| Edge {
            u: *node_map.get_by_left(&e.u).unwrap(),
            v: *node_map.get_by_left(&e.v).unwrap(),
            weights: e.weights.clone(),
            hidden_weights: e.hidden_weights.clone(),
        })
        .collect::<Vec<_>>();

    let g =
        Graph::<Vec<()>, WeightsTuple, Directed>::from_edges(translated_edges.iter().map(|e| {
            (
                NodeIndex::new(e.u),
                NodeIndex::new(e.v),
                WeightsTuple {
                    weights: e.weights.clone().0,
                    hidden_weights: e.hidden_weights.clone().map(|w| w.0).unwrap_or(vec![]),
                },
            )
        }));
    Ok((g, node_map))
}

// Like read_graph_unresetted, but the node ids must be integers from 0 to n-1, where n is the
// number of nodes in the graph. This function is faster than read_graph_unresetted.
pub fn read_graph_with_int_ids(path: &str) -> Result<MLCGraph<()>, Box<dyn Error>> {
    // let mut rdr = csv::Reader::from_path(path)?;
    let mut rdr = csv::ReaderBuilder::new().quote(b'"').from_path(path)?;

    let mut edges = Vec::new();
    for result in rdr.deserialize() {
        let edge: Edge = result?;
        edges.push(edge);
    }

    let g = Graph::<Vec<()>, WeightsTuple, Directed>::from_edges(edges.iter().map(|e| {
        (
            NodeIndex::new(e.u),
            NodeIndex::new(e.v),
            WeightsTuple {
                weights: e.weights.clone().0,
                hidden_weights: e.hidden_weights.clone().map(|w| w.0).unwrap_or(vec![]),
            },
        )
    }));
    Ok(g)
}

impl FromStr for Weights {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = &s[1..s.len() - 1];
        let values = s
            .split(';')
            .map(|s| s.parse::<u64>())
            .collect::<Result<Vec<u64>, _>>()?;
        Ok(Weights(values))
    }
}

impl<'de> Deserialize<'de> for Weights {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}
