use std::{mem::ManuallyDrop, collections::HashSet};

use crate::{
    RBTree,
    binary_tree::BinaryTree,
    node::{Key, NodePtr, Value},
};

pub struct RBTreeIntoIter<K: Key, V: Value> {
    ptr: NodePtr<K, V>,
    rb_tree: ManuallyDrop<RBTree<K, V>>,
    consumed_nodes: HashSet<NodePtr<K, V>>,
}

impl<K: Key, V: Value> Iterator for RBTreeIntoIter<K, V> {
    type Item = (K, V);
    fn next(&mut self) -> Option<Self::Item> {
        if self.rb_tree.is_nil(self.ptr) {
            return None;
        }

        let next = self.rb_tree.inorder_successor(self.ptr);

        unsafe {
            let key_wrapper = std::ptr::read(self.ptr.as_ref().key.assume_init_ref());
            let value_wrapper = std::ptr::read(self.ptr.as_ref().value.assume_init_ref());
            let key = ManuallyDrop::into_inner(key_wrapper);
            let value = ManuallyDrop::into_inner(value_wrapper);

            // Track this node as consumed
            self.consumed_nodes.insert(self.ptr);
            self.ptr = next;
            Some((key, value))
        }
    }
}

impl<K: Key, V: Value> Drop for RBTreeIntoIter<K, V> {
    fn drop(&mut self) {
        // Manually clean up all nodes, handling consumed vs unconsumed differently
        let mut all_nodes = vec![];
        self.rb_tree.traverse(|node| {
            all_nodes.push(node);
        });

        // Clean up data nodes
        for node in all_nodes {
            unsafe {
                let mut b = Box::from_raw(node.as_ptr());
                
                // Check if this node's key/value were consumed during iteration
                if self.consumed_nodes.contains(&node) {
                    // Key/value already moved out, just drop the box
                    // Don't try to drop the ManuallyDrop contents
                } else {
                    // Key/value still present, drop them properly
                    if b.key.as_ptr().is_null() == false {
                        ManuallyDrop::drop(b.key.assume_init_mut());
                    }
                    if b.value.as_ptr().is_null() == false {
                        ManuallyDrop::drop(b.value.assume_init_mut());
                    }
                }
                drop(b);
            }
        }

        // Clean up sentinel nodes
        unsafe {
            drop(Box::from_raw(self.rb_tree.header.as_ptr()));
            drop(Box::from_raw(self.rb_tree.nil.as_ptr()));
        }
    }
}

impl<K: Key, V: Value> IntoIterator for RBTree<K, V> {
    type Item = (K, V);
    type IntoIter = RBTreeIntoIter<K, V>;
    fn into_iter(self) -> Self::IntoIter {
        let first = self.inorder_successor(self.header);

        RBTreeIntoIter {
            ptr: first,
            rb_tree: ManuallyDrop::new(self),
            consumed_nodes: HashSet::new(),
        }
    }
}

pub struct RBTreeIter<'a, K: Key, V: Value> {
    ptr: NodePtr<K, V>,
    rb_tree_ref: &'a RBTree<K, V>,
}

impl<'a, K: Key, V: Value> Iterator for RBTreeIter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        if self.rb_tree_ref.is_nil(self.ptr) {
            return None;
        }

        let next = self.rb_tree_ref.inorder_successor(self.ptr);

        unsafe {
            let key = self.ptr.as_ref().key();
            let value = self.ptr.as_ref().value();

            self.ptr = next;
            Some((key, value))
        }
    }
}

pub struct RBTreeIterMut<'a, K: Key, V: Value> {
    ptr: NodePtr<K, V>,
    rb_tree_mut: &'a mut RBTree<K, V>,
}

impl<'a, K: Key, V: Value> Iterator for RBTreeIterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        if self.rb_tree_mut.is_nil(self.ptr) {
            return None;
        }

        let next = self.rb_tree_mut.inorder_successor(self.ptr);

        unsafe {
            let key = self.ptr.as_ref().key();
            let value = self.ptr.as_mut().value_mut();

            self.ptr = next;
            Some((key, value))
        }
    }
}

impl<'a, K: Key, V: Value> IntoIterator for &'a RBTree<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = RBTreeIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let first = self.inorder_successor(self.header);

        RBTreeIter {
            ptr: first,
            rb_tree_ref: self,
        }
    }
}

impl<'a, K: Key, V: Value> IntoIterator for &'a mut RBTree<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = RBTreeIterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        let first = self.inorder_successor(self.header);

        RBTreeIterMut {
            ptr: first,
            rb_tree_mut: self,
        }
    }
}

impl<K: Key, V: Value> RBTree<K, V> {
    pub fn iter(&self) -> RBTreeIter<'_, K, V> {
        let first = self.inorder_successor(self.header);

        RBTreeIter {
            ptr: first,
            rb_tree_ref: self,
        }
    }

    pub fn iter_mut(&mut self) -> RBTreeIterMut<'_, K, V> {
        let first = self.inorder_successor(self.header);

        RBTreeIterMut {
            ptr: first,
            rb_tree_mut: self,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::RBTree;

    fn setup_tree() -> RBTree<i32, &'static str> {
        let mut tree = RBTree::new();
        tree.insert(10, "ten");
        tree.insert(5, "five");
        tree.insert(15, "fifteen");
        tree.insert(3, "three");
        tree.insert(7, "seven");
        tree.insert(12, "twelve");
        tree.insert(18, "eighteen");
        tree
    }

    #[test]
    fn test_into_iter() {
        let tree = setup_tree();
        let mut items = vec![];
        for item in tree {
            items.push(item);
        }
        assert_eq!(
            items,
            &[
                (3, "three"),
                (5, "five"),
                (7, "seven"),
                (10, "ten"),
                (12, "twelve"),
                (15, "fifteen"),
                (18, "eighteen")
            ]
        );
    }

    #[test]
    fn test_iter() {
        let tree = setup_tree();
        let mut items = vec![];
        for item in &tree {
            items.push(item);
        }
        assert_eq!(
            items,
            &[
                (&3, &"three"),
                (&5, &"five"),
                (&7, &"seven"),
                (&10, &"ten"),
                (&12, &"twelve"),
                (&15, &"fifteen"),
                (&18, &"eighteen")
            ]
        );
    }

    #[test]
    fn test_iter_mut() {
        let mut tree = setup_tree();

        let mut items = vec![];
        for item in &mut tree {
            items.push(item);
        }
        assert_eq!(
            items,
            &[
                (&3, &mut "three"),
                (&5, &mut "five"),
                (&7, &mut "seven"),
                (&10, &mut "ten"),
                (&12, &mut "twelve"),
                (&15, &mut "fifteen"),
                (&18, &mut "eighteen")
            ]
        );

        for (k, v) in &mut tree {
            if *k == 10 {
                *v = "I'm ROOT";
            }
        }

        assert_eq!(tree.get(&10), Some(&"I'm ROOT"));
    }

    #[test]
    fn test_into_iter_early_termination() {
        // Test that memory is properly cleaned up even if iterator is dropped early
        let tree = setup_tree();
        let mut iter = tree.into_iter();
        
        // Only consume first two elements
        let first = iter.next().unwrap();
        let second = iter.next().unwrap();
        
        assert_eq!(first, (3, "three"));
        assert_eq!(second, (5, "five"));
        
        // Drop the iterator early - this should not leak memory
        drop(iter);
        
        // If we get here without segfault/panic, the test passes
    }
}
