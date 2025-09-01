use std::{
    borrow::Borrow,
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};

use crate::{
    binary_search_tree::{BinarySearchTree as BSTTrait, InsertResult},
    binary_tree::{BinaryTree, NodePosition},
    node::{Color, Key, NodePtr, RBNode, Value},
};

#[derive(Debug)]
pub struct BinarySearchTree<K: Key, V: Value> {
    header: NodePtr<K, V>,
    nil: NodePtr<K, V>,
    len: usize,
}

impl<K: Key, V: Value> BinarySearchTree<K, V> {
    pub fn new() -> Self {
        let mut nil_node = Box::new(RBNode {
            key: MaybeUninit::uninit(),
            value: MaybeUninit::uninit(),
            color: Color::Black,
            left: NonNull::dangling(),
            right: NonNull::dangling(),
            parent: NonNull::dangling(),
        });

        let nil_ptr = NonNull::from(&mut *nil_node);
        nil_node.parent = nil_ptr;
        nil_node.left = nil_ptr;
        nil_node.right = nil_ptr;

        let leaked_nil_ptr = NonNull::from(Box::leak(nil_node));

        let header_node = Box::new(RBNode {
            key: MaybeUninit::uninit(),
            value: MaybeUninit::uninit(),
            color: Color::Black,
            left: leaked_nil_ptr,
            right: leaked_nil_ptr,
            parent: leaked_nil_ptr,
        });
        let leaked_header_ptr = NonNull::from(Box::leak(header_node));

        Self {
            header: leaked_header_ptr,
            nil: leaked_nil_ptr,
            len: 0,
        }
    }

    fn is_nil(&self, node: NodePtr<K, V>) -> bool {
        self.nil == node
    }

    fn is_header(&self, node: NodePtr<K, V>) -> bool {
        self.header == node
    }

    fn new_node(&self, key: K, value: V) -> NodePtr<K, V> {
        let node = Box::new(RBNode {
            key: MaybeUninit::new(ManuallyDrop::new(key)),
            value: MaybeUninit::new(ManuallyDrop::new(value)),
            color: Color::Black, // All nodes are black in a simple BST
            left: self.nil,
            right: self.nil,
            parent: self.nil,
        });

        NonNull::from(Box::leak(node))
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.bs_insert(key, value) {
            InsertResult::Old(old_value) => Some(old_value),
            InsertResult::New(_) => {
                self.len += 1;
                None
            }
        }
    }

    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let node = self.bs_remove(key);
        if self.is_nil(node) {
            None
        } else {
            self.len -= 1;
            unsafe {
                let key = ManuallyDrop::into_inner(node.as_ref().key.assume_init_read());
                let value = ManuallyDrop::into_inner(node.as_ref().value.assume_init_read());
                let _ = Box::from_raw(node.as_ptr());
                Some((key, value))
            }
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.search(key)
    }

    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        self.search_mut(key)
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn traverse<F: FnMut(NodePtr<K, V>)>(&self, mut f: F) {
        self._traverse(unsafe { self.header.as_ref().right }, &mut f);
    }

    fn _traverse<F: FnMut(NodePtr<K, V>)>(&self, node: NodePtr<K, V>, f: &mut F) {
        if self.is_nil(node) {
            return;
        }

        self._traverse(unsafe { node.as_ref().left }, f);
        f(node);
        self._traverse(unsafe { node.as_ref().right }, f);
    }

    /// Safe accessor for node key - for public use
    pub fn node_key(&self, node: NodePtr<K, V>) -> &K {
        unsafe { node.as_ref().key() }
    }

    /// Safe accessor for node value - for public use
    pub fn node_value(&self, node: NodePtr<K, V>) -> &V {
        unsafe { node.as_ref().value() }
    }

    /// Traverse the tree in order, calling the closure with key and value
    pub fn traverse_kv<F: FnMut(&K, &V)>(&self, mut f: F) {
        self._traverse_kv(unsafe { self.header.as_ref().right }, &mut f);
    }

    fn _traverse_kv<F: FnMut(&K, &V)>(&self, node: NodePtr<K, V>, f: &mut F) {
        if self.is_nil(node) {
            return;
        }

        self._traverse_kv(unsafe { node.as_ref().left }, f);
        let key = unsafe { node.as_ref().key() };
        let value = unsafe { node.as_ref().value() };
        f(key, value);
        self._traverse_kv(unsafe { node.as_ref().right }, f);
    }
}

// Implement BinaryTree trait
impl<K: Key, V: Value> BinaryTree<K, V> for BinarySearchTree<K, V> {
    fn get_node_position(&self, child: NodePtr<K, V>) -> NodePosition {
        if self.is_nil(child) {
            panic!("child cannot be nil")
        }
        let parent = unsafe { child.as_ref().parent };

        self.get_parent_node_position(parent, child)
    }

    fn get_parent_node_position(
        &self,
        parent: NodePtr<K, V>,
        child: NodePtr<K, V>,
    ) -> NodePosition {
        if self.is_header(parent) {
            return NodePosition::Right;
        }

        let parent_node = unsafe { parent.as_ref() };

        if parent_node.left == child {
            NodePosition::Left
        } else if parent_node.right == child {
            NodePosition::Right
        } else {
            panic!("parent does not point to the child");
        }
    }

    fn inorder_predecessor(&self, node: NodePtr<K, V>) -> NodePtr<K, V> {
        let mut cur = unsafe { node.as_ref().left };

        if self.is_nil(cur) {
            let mut p = unsafe { node.as_ref() }.parent;
            let mut x = node;
            while !self.is_header(p) && x == unsafe { p.as_ref() }.left {
                x = p;
                p = unsafe { p.as_ref() }.parent;
            }

            if self.is_header(p) {
                return self.nil;
            }
            return p;
        }

        loop {
            let right = unsafe { cur.as_ref().right };
            if self.is_nil(right) {
                return cur;
            }
            cur = right;
        }
    }

    fn inorder_successor(&self, node: NodePtr<K, V>) -> NodePtr<K, V> {
        let mut cur = unsafe { node.as_ref().right };

        if self.is_nil(cur) {
            let mut p = unsafe { node.as_ref() }.parent;
            let mut x = node;
            while !self.is_header(p) && x == unsafe { p.as_ref() }.right {
                x = p;
                p = unsafe { p.as_ref() }.parent;
            }

            if self.is_header(p) {
                return self.nil;
            }
            return p;
        }

        loop {
            let left = unsafe { cur.as_ref().left };
            if self.is_nil(left) {
                return cur;
            }
            cur = left;
        }
    }

    fn rotate_left(&mut self, mut node: NodePtr<K, V>) {
        unsafe {
            let mut parent = node.as_ref().parent;

            let mut right = node.as_ref().right;
            if self.is_nil(right) {
                panic!("node without right child cannot rotate left");
            }

            let position = self.get_parent_node_position(parent, node);

            let mut right_left = right.as_ref().left;

            right.as_mut().left = node;
            node.as_mut().parent = right;

            node.as_mut().right = right_left;
            if !self.is_nil(right_left) {
                right_left.as_mut().parent = node;
            }

            match position {
                NodePosition::Left => {
                    parent.as_mut().left = right;
                    right.as_mut().parent = parent;
                }
                NodePosition::Right => {
                    parent.as_mut().right = right;
                    right.as_mut().parent = parent;
                }
            }
        }
    }

    fn rotate_right(&mut self, mut node: NodePtr<K, V>) {
        unsafe {
            let mut parent = node.as_ref().parent;

            let mut left = node.as_ref().left;
            if self.is_nil(left) {
                panic!("node without left child cannot rotate right");
            }

            let position = self.get_parent_node_position(parent, node);

            let mut left_right = left.as_ref().right;

            left.as_mut().right = node;
            node.as_mut().parent = left;

            node.as_mut().left = left_right;
            if !self.is_nil(left_right) {
                left_right.as_mut().parent = node;
            }

            match position {
                NodePosition::Left => {
                    parent.as_mut().left = left;
                    left.as_mut().parent = parent;
                }
                NodePosition::Right => {
                    parent.as_mut().right = left;
                    left.as_mut().parent = parent;
                }
            }
        }
    }

    fn grandparent(&self, node: NodePtr<K, V>) -> NodePtr<K, V> {
        unsafe { node.as_ref().parent.as_ref().parent }
    }

    fn sibling(&self, node: NodePtr<K, V>) -> NodePtr<K, V> {
        unsafe {
            let parent = node.as_ref().parent;
            self.sibling_of_nil(parent, node)
        }
    }

    fn uncle(&self, node: NodePtr<K, V>) -> NodePtr<K, V> {
        unsafe {
            let parent = node.as_ref().parent;
            let grandparent = parent.as_ref().parent;
            self.sibling_of_nil(grandparent, parent)
        }
    }

    fn sibling_of_nil(&self, parent: NodePtr<K, V>, node: NodePtr<K, V>) -> NodePtr<K, V> {
        if self.is_header(parent) {
            return self.nil;
        }
        let parent_node = unsafe { parent.as_ref() };
        if parent_node.left == node {
            parent_node.right
        } else {
            parent_node.left
        }
    }
}

// Implement BinarySearchTree trait
impl<K: Key, V: Value> BSTTrait<K, V> for BinarySearchTree<K, V> {
    fn search<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let mut cur: NodePtr<K, V> = unsafe { self.header.as_ref().right };

        while !self.is_nil(cur) {
            let cur_node = unsafe { cur.as_ref() };

            let k = unsafe { cur_node.key() };

            if key == k.borrow() {
                return unsafe { Some(cur_node.value.assume_init_ref()) };
            }

            if key < k.borrow() {
                cur = cur_node.left;
            } else {
                cur = cur_node.right;
            }
        }

        None
    }

    fn search_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let mut cur: NodePtr<K, V> = unsafe { self.header.as_ref().right };

        while !self.is_nil(cur) {
            let cur_node = unsafe { cur.as_ref() };

            let k = unsafe { cur_node.key().borrow() };

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
            let cur_mut = unsafe { cur.as_mut() };
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
            new_node.as_mut().parent = parent;

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

    fn bs_remove<Q: ?Sized>(&mut self, key: &Q) -> NodePtr<K, V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        let mut cur: NodePtr<K, V> = unsafe { self.header.as_ref().right };

        while !self.is_nil(cur) {
            let cur_mut = unsafe { cur.as_mut() };

            let k = unsafe { cur_mut.key().borrow() };

            if k == key {
                let mut node_to_remove = cur;

                if !self.is_nil(unsafe { node_to_remove.as_ref().left })
                    && !self.is_nil(unsafe { node_to_remove.as_ref().right })
                {
                    // let the in-order predecessor replace it
                    let mut inorder_predecessor = self.inorder_predecessor(cur);

                    unsafe {
                        std::mem::swap(inorder_predecessor.as_mut().key_mut(), cur_mut.key_mut());
                        std::mem::swap(
                            inorder_predecessor.as_mut().value_mut(),
                            cur_mut.value_mut(),
                        );
                    }

                    node_to_remove = inorder_predecessor;
                }

                self.remove_node_with_no_or_one_child(node_to_remove);

                return node_to_remove;
            }

            if key < k {
                cur = cur_mut.left;
            } else {
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
            unsafe {
                let mut parent = node.as_ref().parent;
                let left = node.as_ref().left;
                let right = node.as_ref().right;

                let mut child = if self.is_nil(left) { right } else { left };

                match self.get_parent_node_position(parent, node) {
                    NodePosition::Left => {
                        parent.as_mut().left = child;
                        if !self.is_nil(child) {
                            child.as_mut().parent = parent;
                        }
                    }
                    NodePosition::Right => {
                        parent.as_mut().right = child;
                        if !self.is_nil(child) {
                            child.as_mut().parent = parent;
                        }
                    }
                }
            }
        }
    }
}

// Implement Drop for proper cleanup
impl<K: Key, V: Value> Drop for BinarySearchTree<K, V> {
    fn drop(&mut self) {
        // Collect all nodes first to avoid borrowing issues
        let mut nodes_to_drop = Vec::new();
        
        self.traverse(|node| {
            nodes_to_drop.push(node);
        });
        
        // Drop all nodes
        for node in nodes_to_drop {
            unsafe {
                let node_ref = node.as_ref();
                // Drop the key and value manually
                ManuallyDrop::drop(&mut node_ref.key.assume_init_read());
                ManuallyDrop::drop(&mut node_ref.value.assume_init_read());
                // Drop the box
                let _ = Box::from_raw(node.as_ptr());
            }
        }
        
        // Drop sentinel nodes
        unsafe {
            let _ = Box::from_raw(self.nil.as_ptr());
            let _ = Box::from_raw(self.header.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_bst_operations() {
        let mut bst = BinarySearchTree::new();
        assert!(bst.is_empty());
        assert_eq!(bst.len(), 0);

        // Test insertions
        assert_eq!(bst.insert(5, "five"), None);
        assert_eq!(bst.insert(3, "three"), None);
        assert_eq!(bst.insert(7, "seven"), None);
        assert_eq!(bst.insert(2, "two"), None);
        assert_eq!(bst.insert(4, "four"), None);
        
        assert_eq!(bst.len(), 5);
        assert!(!bst.is_empty());

        // Test search
        assert_eq!(bst.get(&5), Some(&"five"));
        assert_eq!(bst.get(&3), Some(&"three"));
        assert_eq!(bst.get(&7), Some(&"seven"));
        assert_eq!(bst.get(&99), None);

        // Test replacement
        assert_eq!(bst.insert(5, "FIVE"), Some("five"));
        assert_eq!(bst.get(&5), Some(&"FIVE"));
        assert_eq!(bst.len(), 5); // Length should not change

        // Test mutable access
        if let Some(value) = bst.get_mut(&3) {
            *value = "THREE";
        }
        assert_eq!(bst.get(&3), Some(&"THREE"));

        // Test in-order traversal
        let mut traversed = Vec::new();
        bst.traverse_kv(|k, v| {
            traversed.push((*k, *v));
        });
        assert_eq!(traversed, vec![(2, "two"), (3, "THREE"), (4, "four"), (5, "FIVE"), (7, "seven")]);

        // Test removal
        assert_eq!(bst.remove(&3), Some((3, "THREE")));
        assert_eq!(bst.len(), 4);
        assert_eq!(bst.get(&3), None);
        assert_eq!(bst.remove(&99), None); // Non-existent key

        // Test removal of root
        assert_eq!(bst.remove(&5), Some((5, "FIVE")));
        assert_eq!(bst.len(), 3);
        assert_eq!(bst.get(&5), None);

        // Verify remaining elements
        let mut remaining = Vec::new();
        bst.traverse_kv(|k, v| {
            remaining.push((*k, *v));
        });
        assert_eq!(remaining, vec![(2, "two"), (4, "four"), (7, "seven")]);
    }

    #[test]
    fn test_all_nodes_are_black() {
        let mut bst = BinarySearchTree::new();
        bst.insert(5, "five");
        bst.insert(3, "three");
        bst.insert(7, "seven");

        // Verify all nodes are black (since we set all colors to black)
        bst.traverse(|node| {
            let color = unsafe { node.as_ref().color };
            assert_eq!(color, Color::Black);
        });
    }
}
