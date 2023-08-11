mod test;

use bimap::BiMap;
use petgraph::Graph;
use petgraph::{graph::NodeIndex, Undirected};
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

#[derive(Debug)]
struct Edge {
    u: NodeId,
    v: NodeId,
    weights: Weights,
    hidden_weights: Option<Weights>,
}

pub fn read_graph(
    path: &str,
) -> Result<(Graph<(), WeightsTuple, Undirected>, BiMap<String, usize>), Box<dyn Error>> {
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

    let g = Graph::<(), WeightsTuple, Undirected>::from_edges(translated_edges.iter().map(|e| {
        (
            NodeIndex::new(e.u),
            NodeIndex::new(e.v),
            WeightsTuple {
                weights: e.weights.clone().0,
                hidden_weights: e.hidden_weights.clone().map(|w| w.0),
            },
        )
    }));
    return Ok((g, node_map));
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
