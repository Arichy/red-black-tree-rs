use proptest::prelude::*;
use rb_tree::RBTree;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
enum Op<K, V> {
    Insert(K, V),
    Remove(K),
}

proptest! {
    #[test]
    fn fast_differential_test(
        ops in prop::collection::vec(prop_oneof![
            (any::<u16>(), any::<u16>()).prop_map(|(k, v)| Op::Insert(k, v)),
            any::<u16>().prop_map(Op::Remove),
        ], 1..2000)
    ) {
        let mut my_tree = RBTree::new();
        let mut std_tree = BTreeMap::new();

        for (i, op) in ops.iter().enumerate() {
            match op {
                Op::Insert(k, v) => {
                    my_tree.insert(k, v);
                    std_tree.insert(k, v);
                },
                Op::Remove(k) => {
                    my_tree.remove(&k);
                    std_tree.remove(&k);
                }
            }

            if i % 100 == 0 {
                if let Err(e) = my_tree.validate() {
                    panic!("Tree invalid after remove iteration {}: {}", i, e);
                }
            }

            assert_eq!(my_tree.len(), std_tree.len());
        }

        let my_vec: Vec<_> = my_tree.iter().map(|(k, v)| (*k, *v)).collect();
        let std_vec: Vec<_> = std_tree.iter().map(|(k, v)| (*k, *v)).collect();
        assert_eq!(my_vec, std_vec, "Final content mismatch with BTreeMap");

        my_tree.validate().expect("Final tree structure is invalid");
    }
}
