#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_weakly_dominates() {
        let label1 = Label {
            values: vec![1, 2, 3],
            path: vec![0, 1, 2],
            current: 2,
        };
        let label2 = Label {
            values: vec![1, 2, 3],
            path: vec![0, 1, 2],
            current: 2,
        };
        let label3 = Label {
            values: vec![2, 3, 4],
            path: vec![0, 1, 2],
            current: 2,
        };
        let label4 = Label {
            values: vec![1, 2, 4],
            path: vec![0, 1, 2],
            current: 2,
        };

        assert!(label1.weakly_dominates(&label2));
        assert!(label2.weakly_dominates(&label1));
        assert!(!label1.weakly_dominates(&label3));
        assert!(label3.weakly_dominates(&label1));
        assert!(!label1.weakly_dominates(&label4));
        assert!(label4.weakly_dominates(&label1));
    }

    #[test]
    fn test_bag_add_if_necessary() {
        let mut bag = Bag::new_empty();
        let label1 = Label {
            values: vec![1, 2, 3],
            path: vec![0, 1, 2],
            current: 2,
        };
        let label2 = Label {
            values: vec![2, 3, 4],
            path: vec![0, 1, 2],
            current: 2,
        };

        assert!(bag.add_if_necessary(label1.clone()));
        assert_eq!(bag.labels.len(), 1);

        assert!(bag.add_if_necessary(label2.clone()));
        assert_eq!(bag.labels.len(), 1);

        let label3 = Label {
            values: vec![0, 1, 6],
            path: vec![0, 1, 2],
            current: 2,
        };
        assert!(bag.add_if_necessary(label3.clone()));
        assert_eq!(bag.labels.len(), 2);

        assert!(bag.content_dominates(&label1));
        assert!(bag.content_dominates(&label2)); // weakly
        assert!(bag.content_dominates(&label3)); // weakly
    }

    #[test]
    fn test_bag_remove_dominated_by() {
        let mut bag = Bag::new_empty();
        let label1 = Label {
            values: vec![1, 2, 5],
            path: vec![0, 1, 2],
            current: 2,
        };
        let label2 = Label {
            values: vec![2, 3, 4],
            path: vec![0, 1, 2],
            current: 2,
        };
        let label3 = Label {
            values: vec![10, 10, 10],
            path: vec![0, 1, 2],
            current: 2,
        };

        bag.labels.insert(label1.clone());
        bag.labels.insert(label2.clone());
        assert_eq!(bag.labels.len(), 2);

        bag.remove_dominated_by(&label3);
        assert_eq!(bag.labels.len(), 0);
    }
}
