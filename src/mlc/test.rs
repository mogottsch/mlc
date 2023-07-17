#[cfg(test)]
mod tests {
    use crate::mlc;
    use crate::read;

    #[test]
    fn test_run_mlc() {
        let g = read::read_graph("testdata/edges.csv").unwrap();

        let mlc = mlc::MLC::new(g.clone()).unwrap();
        let bags = mlc.run(0);
        let expected_result = mlc::read_bags("testdata/results.csv").unwrap();
        assert!(bags == expected_result);
    }
}
