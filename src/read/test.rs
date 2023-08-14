#[cfg(test)]
mod tests {

    use bimap::BiMap;

    use crate::read;

    #[test]
    fn test_read_graph() {
        let (_, node_map) = read::read_graph_and_reset_ids("testdata/edges_high_index.csv").unwrap();
        let expected_node_map = BiMap::from_iter(vec![
            ("10".to_string(), 0),
            ("11".to_string(), 1),
            ("20".to_string(), 2),
            ("22".to_string(), 3),
        ]);
        assert_eq!(node_map, expected_node_map);
    }
}
