use bag::WeightsTuple;
use color_eyre::eyre::Result;
use log::info;
use petgraph::dot::{Config, Dot};
use petgraph::Graph;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::time::Instant;

pub mod bag;
pub mod mlc;
pub mod read;

#[allow(dead_code)]
fn render_graph(g: &Graph<(), WeightsTuple>, path: &str) -> Result<(), Box<dyn Error>> {
    // save to file
    let mut file = File::create(path)?;
    let dot = Dot::with_config(&g, &[Config::NodeNoLabel]);
    file.write_all(format!("{:?}", dot).as_bytes())?;
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    run_mlc();
    Ok(())
}

#[allow(dead_code)]
fn run_mlc() {
    let path = std::env::args().nth(1).unwrap();
    // let path = "/home/moritz/dev/uni/mcr-py/data/mlc_edges.csv";
    info!("Reading graph from {}", path);
    let g = read::read_graph_with_int_ids(&path).unwrap();

    info!("Creating MLC runner");
    #[allow(unused_mut)]
    let mut mlc = mlc::MLC::new(&g).unwrap();
    // mlc.set_update_label_func(update_label_func);
    // mlc.set_debug(true);
    info!("Running MLC");
    let start = Instant::now();
    #[allow(unused_variables)]
    mlc.set_start_node(0);
    let bags = mlc.run().unwrap();
    let end = Instant::now();
    info!("MLC took {}ms", (end - start).as_millis());
    mlc::write_bags(&bags, "data/labels.csv").unwrap();
}
