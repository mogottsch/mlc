#[cfg(test)]
mod tests {

    use bimap::BiMap;

    use crate::read;

    #[test]
    fn test_run_mlc() {
        let (_, node_map) = read::read_graph("testdata/edges_high_index.csv").unwrap();
        let expected_node_map = BiMap::from_iter(vec![(10, 0), (11, 1), (20, 2), (22, 3)]);

        assert_eq!(node_map, expected_node_map);
    }
}
