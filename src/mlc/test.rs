#[cfg(test)]
mod tests {
    use crate::mlc;
    use crate::read;

    #[test]
    fn test_run_mlc() {
        let g = read::read_graph_with_int_ids("testdata/edges.csv").unwrap();

        let mut mlc = mlc::MLC::new(&g).unwrap();
        mlc.set_start_node(0);
        let bags = mlc.run().unwrap();
        let expected_result = mlc::read_bags("testdata/results.csv").unwrap();
        assert!(bags == &expected_result);
    }
}
