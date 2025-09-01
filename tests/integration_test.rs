use rb_tree::RBTree;

#[test]
fn test_new_tree_is_valid() {
    let tree: RBTree<i32, &str> = RBTree::new();
    if let Err(e) = tree.validate() {
        panic!("Newly created tree is invalid: {}", e);
    }
}

#[test]
fn test_insert_single_node() {
    let mut tree = RBTree::new();
    tree.insert(10, "ten");
    if let Err(e) = tree.validate() {
        panic!("Tree invalid after single insert: {}", e);
    }
}

#[test]
fn test_insert_multiple_nodes_and_validate() {
    let mut tree = RBTree::new();
    let keys = [10, 85, 15, 70, 20, 60, 30, 50, 65, 80, 90, 40, 5, 55];

    for &key in &keys {
        tree.insert(key, "");
        if let Err(e) = tree.validate() {
            panic!("Tree invalid after inserting {}: {}", key, e);
        }
    }
}

#[test]
fn test_insert_triggers_uncle_is_red_case() {
    let mut tree = RBTree::new();
    tree.insert(10, "");
    tree.insert(5, "");
    tree.insert(15, "");
    if let Err(e) = tree.validate() {
        panic!("Tree invalid before final insert: {}", e);
    }
    // This insertion will cause a red-red conflict with a red uncle.
    // N=3, P=5(red), G=10, U=15(red)
    tree.insert(3, "");
    if let Err(e) = tree.validate() {
        panic!("Tree invalid after uncle-is-red case: {}", e);
    }
}

#[test]
fn test_insert_triggers_uncle_is_black_straight_line_case() {
    let mut tree = RBTree::new();
    tree.insert(10, "");
    tree.insert(5, "");
    if let Err(e) = tree.validate() {
        panic!("Tree invalid before final insert: {}", e);
    }
    // This insertion will cause a red-red conflict with a black uncle (nil).
    // It's a left-left case.
    tree.insert(3, "");
    if let Err(e) = tree.validate() {
        panic!("Tree invalid after uncle-is-black-straight case: {}", e);
    }
}

#[test]
fn test_insert_triggers_uncle_is_black_broken_line_case() {
    let mut tree = RBTree::new();
    tree.insert(10, "");
    tree.insert(5, "");
    if let Err(e) = tree.validate() {
        panic!("Tree invalid before final insert: {}", e);
    }
    // This insertion will cause a red-red conflict with a black uncle (nil).
    // It's a left-right case.
    tree.insert(7, "");
    if let Err(e) = tree.validate() {
        panic!("Tree invalid after uncle-is-black-broken case: {}", e);
    }
}

#[test]
fn test_remove_nodes_and_validate() {
    let mut tree = RBTree::new();
    let keys = [10, 85, 15, 70, 20, 60, 30, 50, 65, 80, 90, 40, 5, 55];
    for &key in &keys {
        tree.insert(key, "value");
    }
    if let Err(e) = tree.validate() {
        panic!("Tree invalid after initial insertions: {}", e);
    }

    let keys_to_remove = [30, 10, 70, 90, 55, 5, 85, 15, 20, 60, 50, 65, 80, 40];
    for &key in &keys_to_remove {
        let val = tree.remove(&key);

        assert!(val.is_some(), "Should find and remove key {}", key);
        if let Err(e) = tree.validate() {
            panic!("Tree invalid after removing {}: {}", key, e);
        }
    }

    let val = tree.remove(&keys_to_remove[0]);
    assert!(val.is_none());
}

#[test]
fn test_remove_non_existent() {
    let mut tree = RBTree::new();
    tree.insert(10, "ten");
    let val = tree.remove(&20);
    assert!(val.is_none());
    if let Err(e) = tree.validate() {
        panic!("Tree invalid after removing non-existent key: {}", e);
    }
}

// Additional BST-specific integration tests

#[test]
fn test_bst_property_maintained_during_insertions() {
    let mut tree = RBTree::new();
    let keys_to_insert = [50, 30, 70, 20, 40, 60, 80, 10, 25, 35, 45, 55, 65, 75, 85];
    let mut inserted_keys = Vec::new();

    for &key in &keys_to_insert {
        tree.insert(key, format!("value_{}", key));
        inserted_keys.push(key);

        // Validate BST property after each insertion
        if let Err(e) = tree.validate() {
            panic!("BST property violated after inserting {}: {}", key, e);
        }

        // Verify all previously inserted keys are still searchable
        for &prev_key in &inserted_keys {
            match tree.get(&prev_key) {
                Some(value) => assert_eq!(value, &format!("value_{}", prev_key)),
                None => panic!(
                    "Previously inserted key {} not found after inserting {}",
                    prev_key, key
                ),
            }
        }
    }
}

#[test]
fn test_bst_property_maintained_during_deletions() {
    let mut tree = RBTree::new();
    let keys = vec![50, 30, 70, 20, 40, 60, 80, 10, 25, 35, 45, 55, 65, 75, 85];

    // Insert all keys
    for &key in &keys {
        tree.insert(key, format!("value_{}", key));
    }

    if let Err(e) = tree.validate() {
        panic!("Tree invalid after all insertions: {}", e);
    }

    // Remove keys in random order
    let removal_order = [35, 80, 20, 70, 25, 85, 40, 10, 60, 45, 75, 55, 65, 30, 50];
    let mut remaining_keys: std::collections::HashSet<i32> = keys.iter().cloned().collect();

    for &key_to_remove in &removal_order {
        let removed_value = tree.remove(&key_to_remove);
        assert_eq!(removed_value, Some(format!("value_{}", key_to_remove)));
        remaining_keys.remove(&key_to_remove);

        // Validate BST property after each removal
        if let Err(e) = tree.validate() {
            panic!(
                "BST property violated after removing {}: {}",
                key_to_remove, e
            );
        }

        // Verify remaining keys are still searchable
        for &remaining_key in &remaining_keys {
            match tree.get(&remaining_key) {
                Some(value) => assert_eq!(value, &format!("value_{}", remaining_key)),
                None => panic!(
                    "Remaining key {} not found after removing {}",
                    remaining_key, key_to_remove
                ),
            }
        }

        // Verify removed key is no longer searchable
        assert!(
            tree.get(&key_to_remove).is_none(),
            "Removed key {} still found in tree",
            key_to_remove
        );
    }
}

#[test]
fn test_duplicate_key_handling() {
    let mut tree = RBTree::new();

    // Insert initial value
    tree.insert(42, "original");
    assert_eq!(tree.get(&42), Some(&"original"));

    if let Err(e) = tree.validate() {
        panic!("Tree invalid after initial insert: {}", e);
    }

    // Insert duplicate key with different value
    let old_value = tree.insert(42, "updated");
    assert_eq!(old_value, Some("original"));
    assert_eq!(tree.get(&42), Some(&"updated"));

    if let Err(e) = tree.validate() {
        panic!("Tree invalid after duplicate insert: {}", e);
    }

    // Update again
    let old_value = tree.insert(42, "final");
    assert_eq!(old_value, Some("updated"));
    assert_eq!(tree.get(&42), Some(&"final"));

    if let Err(e) = tree.validate() {
        panic!("Tree invalid after second update: {}", e);
    }
}

#[test]
fn test_sequential_insertion_validation() {
    let mut tree = RBTree::new();

    // Test ascending order insertion
    for i in 1..=20 {
        tree.insert(i, format!("value_{}", i));
        if let Err(e) = tree.validate() {
            panic!(
                "Tree invalid after inserting {} in ascending order: {}",
                i, e
            );
        }
    }

    // Clear and test descending order insertion
    let mut tree = RBTree::new();
    for i in (1..=20).rev() {
        tree.insert(i, format!("value_{}", i));
        if let Err(e) = tree.validate() {
            panic!(
                "Tree invalid after inserting {} in descending order: {}",
                i, e
            );
        }
    }
}

#[test]
fn test_edge_case_operations() {
    let mut tree = RBTree::new();

    // Test removing from empty tree
    assert!(tree.remove(&42).is_none());
    if let Err(e) = tree.validate() {
        panic!("Tree invalid after removing from empty tree: {}", e);
    }

    // Test single node operations
    tree.insert(42, "answer");
    if let Err(e) = tree.validate() {
        panic!("Tree invalid after single insert: {}", e);
    }

    assert_eq!(tree.remove(&42), Some("answer"));
    if let Err(e) = tree.validate() {
        panic!("Tree invalid after removing single node: {}", e);
    }

    // Tree should be empty now
    assert!(tree.get(&42).is_none());

    // Insert and remove same key multiple times
    for i in 0..5 {
        tree.insert(100, "test_value");
        if let Err(e) = tree.validate() {
            panic!("Tree invalid after insert iteration {}: {}", i, e);
        }

        assert_eq!(tree.remove(&100), Some("test_value"));
        if let Err(e) = tree.validate() {
            panic!("Tree invalid after remove iteration {}: {}", i, e);
        }
    }
}

#[test]
fn test_large_tree_validation() {
    let mut tree = RBTree::new();
    let mut keys: Vec<i32> = (1..=10000).collect();

    // Shuffle keys to create more interesting tree structure
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    keys.sort_by_key(|&k| {
        let mut hasher = DefaultHasher::new();
        k.hash(&mut hasher);
        hasher.finish()
    });

    // Insert all keys
    for &key in &keys {
        tree.insert(key, format!("large_value_{}", key));

        // Validate every 100 insertions to avoid too much overhead
        if key % 100 == 0 {
            if let Err(e) = tree.validate() {
                panic!("Large tree invalid after inserting up to {}: {}", key, e);
            }
        }
    }

    // Final validation
    if let Err(e) = tree.validate() {
        panic!("Large tree invalid after all insertions: {}", e);
    }

    // Remove half the keys
    for &key in keys.iter().step_by(2) {
        tree.remove(&key);

        // Validate every 100 removals
        if key % 100 == 0 {
            if let Err(e) = tree.validate() {
                panic!("Large tree invalid after removing up to {}: {}", key, e);
            }
        }
    }

    // Final validation after removals
    if let Err(e) = tree.validate() {
        panic!("Large tree invalid after removals: {}", e);
    }
}

#[test]
fn test_mixed_operations_validation() {
    let mut tree = RBTree::new();
    let base_keys = [50, 25, 75, 12, 37, 62, 87, 6, 18, 31, 43, 56, 68, 81, 93];

    // Insert base keys
    for &key in &base_keys {
        tree.insert(key, format!("base_{}", key));
    }

    if let Err(e) = tree.validate() {
        panic!("Tree invalid after base insertions: {}", e);
    }

    // Perform mixed operations
    let operations = [
        ("insert", 45),
        ("remove", 12),
        ("insert", 15),
        ("remove", 87),
        ("insert", 90),
        ("remove", 25),
        ("insert", 20),
        ("remove", 75),
        ("insert", 85),
        ("remove", 37),
        ("insert", 40),
        ("remove", 62),
    ];

    for (op, key) in operations.iter() {
        match *op {
            "insert" => {
                tree.insert(*key, format!("mixed_{}", key));
            }
            "remove" => {
                tree.remove(key);
            }
            _ => unreachable!(),
        }

        if let Err(e) = tree.validate() {
            panic!("Tree invalid after {} {}: {}", op, key, e);
        }
    }
}
