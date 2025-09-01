use crate::{
    RBTree,
    binary_tree::{BinaryTree, NodePosition},
    node::{Color, Key, NodePtr, RBNode, Value},
};

pub mod validate;

pub(crate) enum InsertResult<K: Key, V: Value> {
    Old(V),
    New(NodePtr<K, V>),
}

pub(crate) trait BinarySearchTree<K: Key, V: Value>: BinaryTree<K, V> {
    fn search(&self, key: &K) -> Option<&V>;
    fn search_mut(&mut self, key: &K) -> Option<&mut V>;
    fn bs_insert(&mut self, key: K, value: V) -> InsertResult<K, V>;
    fn bs_remove(&mut self, key: &K) -> NodePtr<K, V>;

    fn remove_node_with_no_or_one_child(&mut self, node_ptr: NodePtr<K, V>);
    fn remove_node_with_no_child(&mut self, node_ptr: NodePtr<K, V>);
    fn remove_node_with_one_child(&mut self, node_ptr: NodePtr<K, V>);
}

impl<K: Key, V: Value> BinarySearchTree<K, V> for RBTree<K, V> {
    fn search(&self, key: &K) -> Option<&V> {
        let mut cur: NodePtr<K, V> = unsafe { self.header.as_ref().right };

        while !self.is_nil(cur) {
            let cur_node = unsafe { cur.as_ref() };

            let k = unsafe { cur_node.key() };

            if key == k {
                return unsafe { Some(cur_node.value.assume_init_ref()) };
            }

            if key < k {
                cur = cur_node.left;
            } else {
                cur = cur_node.right;
            }
        }

        None
    }

    fn search_mut(&mut self, key: &K) -> Option<&mut V> {
        let mut cur: NodePtr<K, V> = unsafe { self.header.as_ref().right };

        while !self.is_nil(cur) {
            let cur_node = unsafe { cur.as_ref() };

            let k = unsafe { cur_node.key() };

            if key == k {
                return unsafe { Some(cur.as_mut().value.assume_init_mut()) };
            }

            if key < k {
                cur = cur_node.left;
            } else {
                cur = cur_node.right;
            }
        }

        None
    }

    fn bs_insert(&mut self, key: K, value: V) -> InsertResult<K, V> {
        let mut parent = self.header;
        let mut cur = unsafe { self.header.as_ref().right };

        let mut node_position = NodePosition::Right;

        while !self.is_nil(cur) {
            let mut cur_mut = unsafe { cur.as_mut() };
            let k = unsafe { cur_mut.key() };

            if &key == k {
                // replace
                let old_value = std::mem::replace(unsafe { cur_mut.value_mut() }, value);

                return InsertResult::Old(old_value);
            }

            if &key < k {
                parent = cur;
                cur = cur_mut.left;
                node_position = NodePosition::Left;
            } else {
                parent = cur;
                cur = cur_mut.right;
                node_position = NodePosition::Right;
            }
        }

        unsafe {
            let mut new_node = self.new_node(key, value);
            unsafe { new_node.as_mut().parent = parent };

            match node_position {
                NodePosition::Left => {
                    parent.as_mut().left = new_node;
                }
                NodePosition::Right => {
                    parent.as_mut().right = new_node;
                }
            }

            InsertResult::New(new_node)
        }
    }

    fn bs_remove(&mut self, key: &K) -> NodePtr<K, V> {
        let mut parent = self.header;
        let mut cur: NodePtr<K, V> = unsafe { parent.as_ref().right };

        while !self.is_nil(cur) {
            let cur_mut = unsafe { cur.as_mut() };

            let k = unsafe { cur_mut.key() };

            if k == key {
                let mut node_to_remove = cur;

                if !self.is_nil(unsafe { node_to_remove.as_ref().left })
                    && !self.is_nil(unsafe { node_to_remove.as_ref().right })
                {
                    // let the in-order predecessor replace it
                    let mut inorder_predecessor = self.inorder_predecessor(cur);

                    unsafe {
                        std::mem::swap(
                            inorder_predecessor.as_mut().key.assume_init_mut(),
                            cur_mut.key.assume_init_mut(),
                        );
                        std::mem::swap(
                            inorder_predecessor.as_mut().value.assume_init_mut(),
                            cur_mut.value.assume_init_mut(),
                        );
                    }

                    // unsafe {
                    //     println!(
                    //         "after swap: cur: {}, inorder:predecessor: {}",
                    //         cur.as_ref().key.assume_init_ref(),
                    //         inorder_predecessor.as_ref().key.assume_init_ref()
                    //     );
                    // };

                    node_to_remove = inorder_predecessor;
                }

                // unsafe {
                //     println!(
                //         "node_to_remove:{}",
                //         node_to_remove.as_ref().key.assume_init_read()
                //     );
                // }

                self.remove_node_with_no_or_one_child(node_to_remove);

                return node_to_remove;
            }

            if key < k {
                parent = cur;
                cur = cur_mut.left;
            } else {
                parent = cur;
                cur = cur_mut.right;
            }
        }

        cur
    }

    fn remove_node_with_no_or_one_child(&mut self, node: NodePtr<K, V>) {
        if !self.is_nil(node) {
            let left = unsafe { node.as_ref().left };
            let right = unsafe { node.as_ref().right };

            match (self.is_nil(left), self.is_nil(right)) {
                (true, true) => self.remove_node_with_no_child(node),
                (false, false) => unreachable!(),
                _ => self.remove_node_with_one_child(node),
            }
        }
    }

    fn remove_node_with_no_child(&mut self, node: NodePtr<K, V>) {
        if !self.is_nil(node) {
            unsafe {
                let mut parent = node.as_ref().parent;
                match self.get_parent_node_position(parent, node) {
                    NodePosition::Left => parent.as_mut().left = self.nil,
                    NodePosition::Right => parent.as_mut().right = self.nil,
                }
            }
        }
    }

    fn remove_node_with_one_child(&mut self, node: NodePtr<K, V>) {
        if !self.is_nil(node) {
            let mut parent = unsafe { node.as_ref().parent };
            let left = unsafe { node.as_ref().left };
            let right = unsafe { node.as_ref().right };

            let mut child = if !self.is_nil(left) {
                left
            } else if !self.is_nil(right) {
                right
            } else {
                panic!("removed node has two children")
            };

            unsafe {
                match self.get_parent_node_position(parent, node) {
                    NodePosition::Left => {
                        parent.as_mut().left = child;
                        child.as_mut().parent = parent;
                    }
                    NodePosition::Right => {
                        parent.as_mut().right = child;
                        child.as_mut().parent = parent;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        RBTree,
        binary_tree::{BinaryTree, NodePosition},
    };

    use super::BinarySearchTree;

    fn setup_tree() -> RBTree<i32, &'static str> {
        let mut tree = RBTree::new();
        tree.bs_insert(10, "ten");
        tree.bs_insert(5, "five");
        tree.bs_insert(15, "fifteen");
        tree.bs_insert(3, "three");
        tree.bs_insert(7, "seven");
        tree.bs_insert(12, "twelve");
        tree.bs_insert(18, "eighteen");
        tree
    }

    #[test]
    fn test_bs_insert_and_search() {
        let mut tree = RBTree::<i32, &str>::new();

        assert!(tree.is_nil(unsafe { tree.header.as_ref().right }));

        tree.bs_insert(10, "ten");
        assert!(!tree.is_nil(unsafe { tree.header.as_ref().right }));
        assert_eq!(tree.search(&10), Some(&"ten"));

        tree.bs_insert(5, "five");
        assert_eq!(tree.search(&5), Some(&"five"));

        tree.bs_insert(15, "fifteen");
        assert_eq!(tree.search(&15), Some(&"fifteen"));

        // Test inserting a duplicate key
        tree.bs_insert(10, "new ten");
        assert_eq!(tree.search(&10), Some(&"new ten"));
        // Test searching for a non-existent key
        assert_eq!(tree.search(&100), None);
    }

    #[test]
    fn test_bs_remove_leaf_node() {
        let mut tree = setup_tree();
        tree.bs_remove(&3);
        assert_eq!(tree.search(&3), None);
        assert_eq!(tree.search(&5), Some(&"five"));
    }

    #[test]
    fn test_bs_remove_node_with_one_child() {
        let mut tree = setup_tree();
        tree.bs_remove(&3); // remove leaf to create a one-child node case
        // Now node 5 has only one child: 7
        tree.bs_remove(&5);
        assert_eq!(tree.search(&5), None);
        assert_eq!(tree.search(&7), Some(&"seven"));
        assert_eq!(tree.search(&10), Some(&"ten"));
    }

    #[test]
    fn test_bs_remove_node_with_two_children() {
        let mut tree = setup_tree();
        tree.bs_remove(&5); // Node 5 has two children, 3 and 7
        assert_eq!(tree.search(&3), Some(&"three"));
        assert_eq!(tree.search(&5), None);
        assert_eq!(tree.search(&7), Some(&"seven"));
    }

    #[test]
    fn test_bs_remove_root() {
        let mut tree = setup_tree();
        tree.bs_remove(&10);
        assert_eq!(tree.search(&10), None);
        assert_eq!(tree.search(&7), Some(&"seven"));
        assert_eq!(unsafe { tree.header.as_ref().right.as_ref().key() }, &7);
    }

    #[test]
    fn test_bs_remove_root_with_no_children() {
        let mut tree = RBTree::new();
        tree.bs_insert(10, "ten");
        tree.bs_remove(&10);
        assert!(tree.is_nil(unsafe { tree.header.as_ref().right }));
    }

    #[test]
    fn test_bs_remove_root_with_one_child() {
        let mut tree = RBTree::new();
        tree.bs_insert(10, "ten");
        tree.bs_insert(5, "five");
        tree.bs_remove(&10);
        assert_eq!(tree.search(&10), None);
        assert_eq!(tree.search(&5), Some(&"five"));
        let root = unsafe { tree.header.as_ref().right };
        assert!(!tree.is_nil(root));
        assert_eq!(unsafe { root.as_ref().key() }, &5);
    }
}
