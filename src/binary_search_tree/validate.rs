use crate::{
    RBTree,
    node::{Key, NodePtr, Value},
};
use std::fmt::Debug;

/// Validation trait for Binary Search Trees
pub(crate) trait BSTValidator<K: Key, V: Value> {
    /// Validates the entire BST structure and properties
    fn validate_bst(&self) -> Result<(), String>;

    /// Validates BST property recursively with bounds
    fn validate_bst_recursive(
        &self,
        node: NodePtr<K, V>,
        min_bound: Option<&K>,
        max_bound: Option<&K>,
    ) -> Result<(), String>;

    /// Validates tree structure integrity (parent-child relationships)
    fn validate_structure(&self) -> Result<(), String>;

    /// Validates that parent-child pointers are consistent
    fn validate_parent_child_consistency(&self, node: NodePtr<K, V>) -> Result<(), String>;

    /// Validates that there are no cycles in the tree
    fn validate_no_cycles(&self) -> Result<(), String>;

    /// Counts nodes in the tree
    fn count_nodes(&self) -> usize;
}

impl<K: Key + Debug, V: Value> BSTValidator<K, V> for RBTree<K, V> {
    fn validate_bst(&self) -> Result<(), String> {
        // First validate the basic structure
        self.validate_structure()?;

        // Then validate BST properties
        let root = unsafe { self.header.as_ref().right };
        if !self.is_nil(root) {
            self.validate_bst_recursive(root, None, None)?;
        }

        // Validate no cycles
        self.validate_no_cycles()?;

        Ok(())
    }

    fn validate_bst_recursive(
        &self,
        node: NodePtr<K, V>,
        min_bound: Option<&K>,
        max_bound: Option<&K>,
    ) -> Result<(), String> {
        if self.is_nil(node) {
            return Ok(());
        }

        let node_ref = unsafe { node.as_ref() };
        let key = unsafe { node_ref.key() };

        // Check if current node violates BST property with bounds
        if let Some(min) = min_bound {
            if key <= min {
                return Err(format!(
                    "BST violation: node key {:?} should be greater than {:?}",
                    key, min
                ));
            }
        }

        if let Some(max) = max_bound {
            if key >= max {
                return Err(format!(
                    "BST violation: node key {:?} should be less than {:?}",
                    key, max
                ));
            }
        }

        // Recursively validate left subtree (all values should be < current key)
        self.validate_bst_recursive(node_ref.left, min_bound, Some(key))?;

        // Recursively validate right subtree (all values should be > current key)
        self.validate_bst_recursive(node_ref.right, Some(key), max_bound)?;

        Ok(())
    }

    fn validate_structure(&self) -> Result<(), String> {
        let root = unsafe { self.header.as_ref().right };

        if self.is_nil(root) {
            // Empty tree is valid
            return Ok(());
        }

        // Validate that root's parent is header
        let root_ref = unsafe { root.as_ref() };
        if root_ref.parent != self.header {
            return Err("Root node's parent should be header".to_string());
        }

        // Validate parent-child consistency for all nodes
        self.validate_parent_child_consistency(root)?;

        Ok(())
    }

    fn validate_parent_child_consistency(&self, node: NodePtr<K, V>) -> Result<(), String> {
        if self.is_nil(node) {
            return Ok(());
        }

        let node_ref = unsafe { node.as_ref() };
        let key = unsafe { node_ref.key.assume_init_ref() };

        // Validate left child
        if !self.is_nil(node_ref.left) {
            let left_ref = unsafe { node_ref.left.as_ref() };
            if left_ref.parent != node {
                return Err(format!(
                    "Parent-child inconsistency: left child of {:?} doesn't point back to parent",
                    key
                ));
            }
            self.validate_parent_child_consistency(node_ref.left)?;
        }

        // Validate right child
        if !self.is_nil(node_ref.right) {
            let right_ref = unsafe { node_ref.right.as_ref() };
            if right_ref.parent != node {
                return Err(format!(
                    "Parent-child inconsistency: right child of {:?} doesn't point back to parent",
                    key
                ));
            }
            self.validate_parent_child_consistency(node_ref.right)?;
        }

        Ok(())
    }

    fn validate_no_cycles(&self) -> Result<(), String> {
        use std::collections::HashSet;
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        let root = unsafe { self.header.as_ref().right };
        if !self.is_nil(root) {
            self.detect_cycle_util(root, &mut visited, &mut rec_stack)?;
        }

        Ok(())
    }

    fn count_nodes(&self) -> usize {
        let mut count = 0;
        self.traverse(|_| count += 1);
        count
    }
}

impl<K: Key + Debug, V: Value> RBTree<K, V> {
    /// Helper method to detect cycles using DFS
    fn detect_cycle_util(
        &self,
        node: NodePtr<K, V>,
        visited: &mut std::collections::HashSet<NodePtr<K, V>>,
        rec_stack: &mut std::collections::HashSet<NodePtr<K, V>>,
    ) -> Result<(), String> {
        if self.is_nil(node) {
            return Ok(());
        }

        if rec_stack.contains(&node) {
            return Err("Cycle detected in tree structure".to_string());
        }

        if visited.contains(&node) {
            return Ok(());
        }

        visited.insert(node);
        rec_stack.insert(node);

        let node_ref = unsafe { node.as_ref() };

        // Check left child
        self.detect_cycle_util(node_ref.left, visited, rec_stack)?;

        // Check right child
        self.detect_cycle_util(node_ref.right, visited, rec_stack)?;

        rec_stack.remove(&node);
        Ok(())
    }

    /// Validates BST property by doing an in-order traversal
    pub fn validate_inorder(&self) -> Result<(), String>
    where
        K: Clone,
    {
        let mut prev_key: Option<K> = None;
        let mut is_valid = true;
        let mut error_msg = String::new();

        self.traverse(|node| {
            if !is_valid {
                return;
            }

            let node_ref = unsafe { node.as_ref() };
            let key = unsafe { node_ref.key() };

            if let Some(ref prev) = prev_key {
                if key <= prev {
                    is_valid = false;
                    error_msg = format!(
                        "BST violation in inorder traversal: {:?} should be greater than {:?}",
                        key, prev
                    );
                    return;
                }
            }

            prev_key = Some(key.clone());
        });

        if is_valid { Ok(()) } else { Err(error_msg) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary_search_tree::BinarySearchTree;

    fn create_test_tree() -> RBTree<i32, String> {
        let mut tree = RBTree::new();
        tree.bs_insert(10, "ten".to_string());
        tree.bs_insert(5, "five".to_string());
        tree.bs_insert(15, "fifteen".to_string());
        tree.bs_insert(3, "three".to_string());
        tree.bs_insert(7, "seven".to_string());
        tree.bs_insert(12, "twelve".to_string());
        tree.bs_insert(18, "eighteen".to_string());
        tree
    }

    #[test]
    fn test_valid_bst() {
        let tree = create_test_tree();
        if let Err(e) = tree.validate_bst() {
            panic!("BST validation failed: {}", e);
        }
    }

    #[test]
    fn test_valid_bst_inorder() {
        let tree = create_test_tree();
        if let Err(e) = tree.validate_inorder() {
            panic!("BST inorder validation failed: {}", e);
        }
    }

    #[test]
    fn test_empty_tree_validation() {
        let tree: RBTree<i32, String> = RBTree::new();
        if let Err(e) = tree.validate_bst() {
            panic!("Empty tree BST validation failed: {}", e);
        }
        if let Err(e) = tree.validate_inorder() {
            panic!("Empty tree inorder validation failed: {}", e);
        }
        assert_eq!(tree.count_nodes(), 0);
    }

    #[test]
    fn test_single_node_tree() {
        let mut tree = RBTree::new();
        tree.bs_insert(42, "answer".to_string());
        if let Err(e) = tree.validate_bst() {
            panic!("Single node tree BST validation failed: {}", e);
        }
        if let Err(e) = tree.validate_inorder() {
            panic!("Single node tree inorder validation failed: {}", e);
        }
        assert_eq!(tree.count_nodes(), 1);
    }

    #[test]
    fn test_count_nodes() {
        let tree = create_test_tree();
        assert_eq!(tree.count_nodes(), 7);
    }

    #[test]
    fn test_structure_validation() {
        let tree = create_test_tree();
        if let Err(e) = tree.validate_structure() {
            panic!("Tree structure validation failed: {}", e);
        }
    }

    #[test]
    fn test_no_cycles() {
        let tree = create_test_tree();
        if let Err(e) = tree.validate_no_cycles() {
            panic!("Cycle detection failed: {}", e);
        }
    }

    #[test]
    fn test_bst_property_with_duplicates() {
        let mut tree = RBTree::new();
        tree.bs_insert(10, "ten".to_string());
        tree.bs_insert(5, "five".to_string());
        tree.bs_insert(15, "fifteen".to_string());
        // Insert duplicate - should replace existing value
        tree.bs_insert(10, "new_ten".to_string());

        if let Err(e) = tree.validate_bst() {
            panic!("BST validation failed with duplicates: {}", e);
        }
        if let Err(e) = tree.validate_inorder() {
            panic!("BST inorder validation failed with duplicates: {}", e);
        }
        assert_eq!(tree.search(&10), Some(&"new_ten".to_string()));
    }

    #[test]
    fn test_complex_tree_validation() {
        let mut tree = RBTree::new();
        let values = [50, 30, 70, 20, 40, 60, 80, 10, 25, 35, 45];

        for &val in &values {
            tree.bs_insert(val, val.to_string());
        }

        if let Err(e) = tree.validate_bst() {
            panic!("Complex tree BST validation failed: {}", e);
        }
        if let Err(e) = tree.validate_inorder() {
            panic!("Complex tree inorder validation failed: {}", e);
        }
        assert_eq!(tree.count_nodes(), values.len());

        // Verify all values can be found
        for &val in &values {
            assert_eq!(tree.search(&val), Some(&val.to_string()));
        }
    }

    #[test]
    fn test_validation_after_removals() {
        let mut tree = create_test_tree();

        // Remove some nodes
        tree.bs_remove(&3);
        tree.bs_remove(&15);

        if let Err(e) = tree.validate_bst() {
            panic!("BST validation failed after removals: {}", e);
        }
        if let Err(e) = tree.validate_inorder() {
            panic!("BST inorder validation failed after removals: {}", e);
        }
        assert_eq!(tree.count_nodes(), 5);
    }

    #[test]
    fn basic_bst() {
        let mut tree = RBTree::new();
        tree.bs_insert(1, "1");
        tree.bs_insert(2, "2");
        tree.bs_insert(3, "3");
        tree.display();

        if let Err(e) = tree.validate_bst() {
            panic!(
                "BST validation failed after initial insertions (1-20): {}",
                e
            );
        }
    }

    #[test]
    fn test_validation_after_multiple_operations() {
        let mut tree = RBTree::new();

        // Insert nodes
        for i in 1..=20 {
            tree.bs_insert(i, i.to_string());
        }

        tree.display();

        if let Err(e) = tree.validate_bst() {
            panic!(
                "BST validation failed after initial insertions (1-20): {}",
                e
            );
        }
        assert_eq!(tree.count_nodes(), 20);

        // Remove some nodes
        for i in (1..=20).step_by(2) {
            tree.bs_remove(&i);
        }

        if let Err(e) = tree.validate_bst() {
            panic!("BST validation failed after removals (odd numbers): {}", e);
        }
        assert_eq!(tree.count_nodes(), 10);

        // Insert different values
        for i in 21..=30 {
            tree.bs_insert(i, i.to_string());
        }

        if let Err(e) = tree.validate_bst() {
            panic!(
                "BST validation failed after final insertions (21-30): {}",
                e
            );
        }
        assert_eq!(tree.count_nodes(), 20);
    }
}
