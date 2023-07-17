use crate::bag::*;
use petgraph::dot::{Config, Dot};
use petgraph::Graph;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

pub mod bag;
pub mod mlc;
pub mod read;

fn render_graph(g: &Graph<u64, Weights>, path: &str) -> Result<(), Box<dyn Error>> {
    // save to file
    let mut file = File::create(path)?;
    let dot = Dot::with_config(&g, &[Config::NodeNoLabel]);
    file.write_all(format!("{:?}", dot).as_bytes())?;
    Ok(())
}

fn main() {
    let bags = mlc::read_bags("testdata/results.csv").unwrap();
    println!("{:#?}", bags);
    // let g = read::read_graph("testdata/edges.csv").unwrap();
    //
    // let mlc = mlc::new(g.clone()).unwrap();
    //
    // let bags = mlc.run(0);
    // println!("{:#?}", bags);
    // bags.iter().for_each(|(k, v)| {
    //     println!("{}: {:?}", k, v.labels.len());
    // });
    //
    // if let Err(e) = render_graph(&g, "data/graph.dot") {
    //     println!("Error rendering graph: {}", e);
    // }
}
