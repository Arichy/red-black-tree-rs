use proptest::prelude::*;
use rb_tree::RBTree;

proptest! {
    #[test]
    fn rb_tree(keys in prop::collection::vec(any::<i32>(), 1..=1000)) {
        let mut tree = RBTree::new();
        for key in &keys {
            tree.insert(*key, *key);
            if let Err(e) = tree.validate() {
                panic!("Tree invalid after initial insertions: {}", e);
            }
        }

        let mut unique_keys: Vec<_> = keys.clone();
        unique_keys.sort();
        unique_keys.dedup();

        for key in &unique_keys {
            assert!(tree.get(key).is_some());
        }


        for (index, key) in unique_keys.iter().enumerate() {
            tree.remove(key);
            if index % 100 == 0 {
                if let Err(e) = tree.validate() {
                    panic!("Tree invalid after removing {}: {}", key, e);
                }
            }
        }
    }
}

#[test]
#[ignore]
fn minimal() {
    let keys = vec![2, 1];
    let mut tree = RBTree::new();
    for key in &keys {
        tree.insert(*key, *key);
        if let Err(e) = tree.validate() {
            panic!("Tree invalid after initial insertions: {}", e);
        }
    }

    tree.display();

    for key in &keys {
        assert!(tree.get(key).is_some());
    }

    for key in &keys {
        tree.remove(key);
        println!("after removing {key}");
        tree.display();
        if let Err(e) = tree.validate() {
            panic!("Tree invalid after removing {}: {}", key, e);
        }
    }
}
