use bag::WeightsTuple;
use color_eyre::eyre::Result;
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
    // let path = "/home/moritz/dev/uni/mcr-py/data/mlc_edges.csv";
    // println!("Reading graph from {}", path);
    // let (g, _) = read::read_graph(&path).unwrap();
    //
    // if let Err(e) = render_graph(&g, "data/graph.dot") {
    //     println!("Error rendering graph: {}", e);
    // }
    Ok(())
}

const PRICE_INCREMENT_INTERVAL: u64 = 3;
fn update_label_func(
    old_label: &bag::Label<usize>,
    new_label: &bag::Label<usize>,
) -> bag::Label<usize> {
    let old_hidden_values = old_label.hidden_values.clone().unwrap();
    let old_bicycle_duration = old_hidden_values[0];
    let old_bicycle_duration_minutes = old_bicycle_duration / 1000 / 60;

    let old_price_increment_intervals = old_bicycle_duration_minutes / PRICE_INCREMENT_INTERVAL;

    let new_hidden_values = new_label.hidden_values.clone().unwrap();
    let new_bicycle_duration = new_hidden_values[0];
    let new_bicycle_duration_minutes = new_bicycle_duration / 1000 / 60;

    let new_price_increment_intervals = new_bicycle_duration_minutes / PRICE_INCREMENT_INTERVAL;

    let price_increment = new_price_increment_intervals - old_price_increment_intervals;
    let new_price = new_label.values[1] + price_increment;
    let new_values = vec![new_label.values[0], new_price];
    bag::Label {
        node_id: new_label.node_id,
        path: new_label.path.clone(),
        values: new_values,
        hidden_values: new_label.hidden_values.clone(),
    }
}
#[allow(dead_code)]
fn run_mlc() {
    let path = std::env::args().nth(1).unwrap();
    // let path = "/home/moritz/dev/uni/mcr-py/data/mlc_edges.csv";
    println!("Reading graph from {}", path);
    let (g, node_map) = read::read_graph(&path).unwrap();

    println!("Creating MLC runner");
    let mlc = mlc::MLC::new(g, node_map, Some(update_label_func)).unwrap();
    println!("Running MLC");
    let start = Instant::now();
    let bags = mlc.run("B11010211890".to_string());
    let end = Instant::now();
    println!("MLC took {}ms", (end - start).as_millis());
    mlc::write_bags(&bags, "data/labels.csv").unwrap();
}
