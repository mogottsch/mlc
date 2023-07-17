use petgraph::graph::NodeIndex;
use petgraph::Graph;
use serde::{de, Deserialize, Deserializer};
use std::{error::Error, str::FromStr};

use crate::bag::{NodeId, Weights};

#[derive(Debug, serde::Deserialize)]
struct Edge {
    u: NodeId,
    v: NodeId,
    weights: Weights,
}

pub fn read_graph(path: &str) -> Result<Graph<u64, Weights>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut edges = Vec::new();
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let edge: Edge = result?;
        edges.push(edge);
    }

    let g = Graph::<u64, Weights>::from_edges(
        edges
            .iter()
            .map(|e| (NodeIndex::new(e.u), NodeIndex::new(e.v), e.weights.clone())),
    );
    return Ok(g);
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
