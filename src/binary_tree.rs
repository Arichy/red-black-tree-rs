use crate::{
    RBTree,
    node::{Key, NodePtr, Value},
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum NodePosition {
    Left,
    Right,
}

pub(crate) trait BinaryTree<K: Key, V: Value> {
    fn get_node_position(&self, child: NodePtr<K, V>) -> NodePosition;
    fn get_parent_node_position(&self, parent: NodePtr<K, V>, child: NodePtr<K, V>)
    -> NodePosition;
    fn inorder_predecessor(&self, node: NodePtr<K, V>) -> NodePtr<K, V>;
    fn inorder_successor(&self, node: NodePtr<K, V>) -> NodePtr<K, V>;
    fn rotate_left(&mut self, node: NodePtr<K, V>);
    fn rotate_right(&mut self, node: NodePtr<K, V>);
    #[allow(dead_code)]
    fn sibling(&self, node: NodePtr<K, V>) -> NodePtr<K, V>;
    fn grandparent(&self, node: NodePtr<K, V>) -> NodePtr<K, V>;
    fn uncle(&self, node: NodePtr<K, V>) -> NodePtr<K, V>;
    fn sibling_of_nil(&self, parent: NodePtr<K, V>, node: NodePtr<K, V>) -> NodePtr<K, V>;
}

impl<K: Key, V: Value> BinaryTree<K, V> for RBTree<K, V> {
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

    //      parent              parent
    //        /                    /
    //     node                  right
    //       \                    /
    //       right              node
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

    //      parent               parent
    //        /                    /
    //     node                  left
    //      /                      \
    //    left                     node
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
            if self.is_header(node) || self.is_header(parent) {
                return self.nil;
            }

            let grandparent = parent.as_ref().parent;

            match self.get_parent_node_position(grandparent, parent) {
                NodePosition::Left => grandparent.as_ref().right,
                NodePosition::Right => grandparent.as_ref().left,
            }
        }
    }

    fn sibling_of_nil(&self, parent: NodePtr<K, V>, node: NodePtr<K, V>) -> NodePtr<K, V> {
        unsafe {
            if self.is_header(parent) {
                return self.nil;
            }
            match self.get_parent_node_position(parent, node) {
                NodePosition::Left => parent.as_ref().right,
                NodePosition::Right => parent.as_ref().left,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        RBTree,
        binary_search_tree::BinarySearchTree,
        binary_tree::{BinaryTree, NodePosition},
    };

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
    fn test_sibling() {
        let tree = setup_tree();
        tree.display();
        let root = unsafe { tree.header.as_ref().right };
        let node_5 = unsafe { root.as_ref().left };
        let node_15 = unsafe { root.as_ref().right };
        assert_eq!(unsafe { tree.sibling(node_5).as_ref().key() }, &15);
        assert_eq!(unsafe { tree.sibling(node_15).as_ref().key() }, &5);

        let node_3 = unsafe { node_5.as_ref().left };
        let node_7 = unsafe { node_5.as_ref().right };
        assert_eq!(unsafe { tree.sibling(node_3).as_ref().key() }, &7);
        assert_eq!(unsafe { tree.sibling(node_7).as_ref().key() }, &3);
    }

    #[test]
    fn test_grandparent() {
        let tree = setup_tree();
        let root = unsafe { tree.header.as_ref().right };
        let node_5 = unsafe { root.as_ref().left };
        let node_3 = unsafe { node_5.as_ref().left };
        let grandparent = tree.grandparent(node_3);
        assert_eq!(unsafe { grandparent.as_ref().key() }, &10);
    }

    #[test]
    fn test_uncle() {
        let tree = setup_tree();
        let root = unsafe { tree.header.as_ref().right };
        let node_5 = unsafe { root.as_ref().left };
        let node_15 = unsafe { root.as_ref().right };
        let node_3 = unsafe { node_5.as_ref().left };
        let uncle = tree.uncle(node_3);
        assert_eq!(unsafe { uncle.as_ref().key() }, &15);

        let node_12 = unsafe { node_15.as_ref().left };
        let uncle = tree.uncle(node_12);
        assert_eq!(unsafe { uncle.as_ref().key() }, &5);
    }

    #[test]
    fn test_rotate_left() {
        let mut tree = setup_tree();
        let root = unsafe { tree.header.as_ref().right };
        tree.rotate_left(root);
        let new_root = unsafe { tree.header.as_ref().right };
        assert_eq!(unsafe { new_root.as_ref().key() }, &15);
        let new_root_left = unsafe { new_root.as_ref().left };
        assert_eq!(unsafe { new_root_left.as_ref().key() }, &10);
        let new_root_left_right = unsafe { new_root_left.as_ref().right };
        assert_eq!(unsafe { new_root_left_right.as_ref().key() }, &12);
    }

    #[test]
    fn test_rotate_right() {
        let mut tree = setup_tree();
        let root = unsafe { tree.header.as_ref().right };
        tree.rotate_right(root);
        let new_root = unsafe { tree.header.as_ref().right };
        assert_eq!(unsafe { new_root.as_ref().key() }, &5);
        let new_root_right = unsafe { new_root.as_ref().right };
        assert_eq!(unsafe { new_root_right.as_ref().key() }, &10);
        let new_root_right_left = unsafe { new_root_right.as_ref().left };
        assert_eq!(unsafe { new_root_right_left.as_ref().key() }, &7);
    }

    #[test]
    fn test_inorder_predecessor() {
        let tree = setup_tree();
        let root = unsafe { tree.header.as_ref().right };
        let predecessor = tree.inorder_predecessor(root);
        assert!(!tree.is_nil(predecessor));
        assert_eq!(unsafe { predecessor.as_ref().key() }, &7);
    }

    #[test]
    fn test_get_parent_node_position() {
        let tree = setup_tree();
        let root = unsafe { tree.header.as_ref().right };
        let root_node = unsafe { root.as_ref() };
        let left_child = root_node.left;
        let right_child = root_node.right;

        assert_eq!(
            tree.get_parent_node_position(tree.header, root),
            NodePosition::Right
        );
        assert_eq!(
            tree.get_parent_node_position(root, left_child),
            NodePosition::Left
        );
        assert_eq!(
            tree.get_parent_node_position(root, right_child),
            NodePosition::Right
        );
    }
}
