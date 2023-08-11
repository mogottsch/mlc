#[cfg(test)]
mod tests {
    use crate::mlc;
    use crate::read;

    #[test]
    fn test_run_mlc() {
        let (g, node_map) = read::read_graph("testdata/edges.csv").unwrap();

        let mlc = mlc::MLC::new(g,node_map).unwrap();
        let bags = mlc.run_resetted(0);
        let expected_result = mlc::read_bags("testdata/results.csv").unwrap();
        assert!(bags == expected_result);
    }
}
