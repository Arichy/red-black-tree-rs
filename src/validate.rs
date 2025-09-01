use std::fmt::{Debug, Display};

use crate::{
    RBTree,
    binary_search_tree::validate::BSTValidator,
    node::{Color, Key, NodePtr, Value},
};

#[derive(Debug, PartialEq, Eq)]
pub enum RBTreeError<K: Key> {
    /// property 2: root is not black
    RootNotBlack { root: K },
    /// property 4: red node has a red child
    RedParentRedChild { parent: K, child: K },
    /// property 5: black height mismatch
    BlackHeightMismatch {
        node: K,
        left_b_height: usize,
        right_b_height: usize,
    },
    /// BST property violation
    BSTViolation { message: String },
}

impl<K: Key + Display> Display for RBTreeError<K> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RBTreeError::RootNotBlack { root } => {
                write!(
                    f,
                    "Red-Black Tree validation failed: Root node '{}' is not black",
                    root
                )
            }
            RBTreeError::RedParentRedChild { parent, child } => {
                write!(
                    f,
                    "Red-Black Tree validation failed: Red parent node '{}' has red child '{}'",
                    parent, child
                )
            }
            RBTreeError::BlackHeightMismatch {
                node,
                left_b_height,
                right_b_height,
            } => {
                write!(
                    f,
                    "Red-Black Tree validation failed: Black height mismatch at node '{}' (left: {}, right: {})",
                    node, left_b_height, right_b_height
                )
            }
            RBTreeError::BSTViolation { message } => {
                write!(f, "Binary Search Tree validation failed: {}", message)
            }
        }
    }
}

impl<K: Key + Clone + Debug, V: Value + Clone> RBTree<K, V> {
    pub fn validate(&self) -> Result<(), RBTreeError<K>> {
        // First validate BST properties using the trait
        if let Err(bst_error) = BSTValidator::validate_bst(self) {
            return Err(RBTreeError::BSTViolation { message: bst_error });
        }

        let root = unsafe { self.header.as_ref().right };
        if self.is_nil(root) {
            return Ok(());
        }

        // property 2: root is black
        if unsafe { root.as_ref() }.color == Color::Red {
            return Err(RBTreeError::RootNotBlack {
                root: unsafe { root.as_ref().key() }.clone(),
            });
        }

        // property 4 & 5
        self.validate_subtree(root)?;

        Ok(())
    }

    fn validate_subtree(&self, node: NodePtr<K, V>) -> Result<usize, RBTreeError<K>> {
        if self.is_nil(node) {
            return Ok(1); // black height of nil is 1
        }

        let node_ref = unsafe { node.as_ref() };

        // property 4: red node cannot have red children
        if node_ref.color == Color::Red {
            let left_child = unsafe { node_ref.left.as_ref() };
            if left_child.color == Color::Red {
                return Err(RBTreeError::RedParentRedChild {
                    parent: unsafe { node_ref.key() }.clone(),
                    child: unsafe { left_child.key() }.clone(),
                });
            }

            let right_child = unsafe { node_ref.right.as_ref() };
            if right_child.color == Color::Red {
                return Err(RBTreeError::RedParentRedChild {
                    parent: unsafe { node_ref.key() }.clone(),
                    child: unsafe { right_child.key() }.clone(),
                });
            }
        }

        let left_b_height = self.validate_subtree(node_ref.left)?;
        let right_b_height = self.validate_subtree(node_ref.right)?;

        // property 5: black height must be same for all paths
        if left_b_height != right_b_height {
            return Err(RBTreeError::BlackHeightMismatch {
                node: unsafe { node_ref.key() }.clone(),
                left_b_height,
                right_b_height,
            });
        }

        let self_b_height = left_b_height + if node_ref.color == Color::Black { 1 } else { 0 };
        Ok(self_b_height)
    }
}
