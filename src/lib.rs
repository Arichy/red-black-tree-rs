use crate::{
    binary_search_tree::{BinarySearchTree, InsertResult},
    binary_tree::{BinaryTree, NodePosition},
    node::{Color, Key, NodePtr, RBNode, Value},
};
use std::{
    borrow::Borrow,
    fmt::{Debug, Display},
    mem::{ManuallyDrop, MaybeUninit},
    ptr::NonNull,
};

mod binary_search_tree;
mod binary_tree;
mod iter;
mod node;
mod validate;

// Re-export the validation trait for external use
use binary_search_tree::validate::BSTValidator;

// Re-export our simple BinarySearchTree implementation
pub use binary_search_tree::binary_search_tree_impl::BinarySearchTree as SimpleBST;

#[derive(Debug)]
pub struct RBTree<K: Key, V: Value> {
    header: NodePtr<K, V>,
    nil: NodePtr<K, V>,
    len: usize,
}

impl<K: Key, V: Value> RBTree<K, V> {
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
            color: Color::Red,
            left: self.nil,
            right: self.nil,
            parent: self.nil,
        });

        NonNull::from(Box::leak(node))
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

    pub(crate) fn search<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord,
    {
        BinarySearchTree::search(self, key)
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

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.bs_insert(key, value) {
            InsertResult::Old(old_value) => Some(old_value),
            InsertResult::New(red_node) => {
                self.insert_fixup(red_node);
                self.len += 1;
                None
            }
        }
    }

    fn insert_fixup(&mut self, mut red_node: NodePtr<K, V>) {
        let parent = unsafe { red_node.as_ref().parent };
        if self.is_header(parent) {
            unsafe { red_node.as_mut().color = Color::Black };
            return;
        }

        match unsafe { parent.as_ref() }.color {
            Color::Black => {
                // if parent is black, done
                return;
            }
            Color::Red => {
                // if parent is red, resolve red-red conflict
                let grandparent = self.grandparent(red_node);
                // parent is red, so parent must not be root, so parent must have parent, so grandparent must not be nil
                // grandparent must be black
                assert!(!self.is_nil(grandparent));

                // check color of uncle
                let uncle = self.uncle(red_node);
                match unsafe { uncle.as_ref().color } {
                    Color::Black => {
                        // uncle is black
                        //   1. check N-P-G, if it's a broken line, rotate P and turn it to a straight line
                        //   2. if it's a straight line, rotate G, color P to black, color G to red
                        let g_position = self.get_node_position(parent);
                        let n_position = self.get_node_position(red_node);

                        match (g_position, n_position) {
                            (NodePosition::Left, NodePosition::Left) => self
                                .insert_fixup_straight_line(
                                    red_node,
                                    parent,
                                    grandparent,
                                    NodePosition::Left,
                                ),
                            (NodePosition::Right, NodePosition::Right) => self
                                .insert_fixup_straight_line(
                                    red_node,
                                    parent,
                                    grandparent,
                                    NodePosition::Right,
                                ),
                            (NodePosition::Left, NodePosition::Right) => {
                                self.rotate_left(parent);
                                self.insert_fixup_straight_line(
                                    parent,
                                    red_node,
                                    grandparent,
                                    NodePosition::Left,
                                );
                            }
                            (NodePosition::Right, NodePosition::Left) => {
                                self.rotate_right(parent);
                                self.insert_fixup_straight_line(
                                    parent,
                                    red_node,
                                    grandparent,
                                    NodePosition::Right,
                                );
                            }
                        }
                    }
                    Color::Red => {
                        // uncle is red
                        //   1. parent and uncle turn black
                        //   2. grandparent turns red
                        //   3. resolve red-red conflict for grandparent

                        // parent is red,
                        // uncle is red, so uncle must not be nil
                        assert!(!self.is_nil(uncle));

                        self.color_black(parent);
                        self.color_black(uncle);

                        self.color_red(grandparent);

                        self.insert_fixup(grandparent);
                    }
                }
            }
        }
    }

    fn insert_fixup_straight_line(
        &mut self,
        red_child: NodePtr<K, V>,
        red_p: NodePtr<K, V>,
        black_g: NodePtr<K, V>,
        position: NodePosition,
    ) {
        assert_eq!(unsafe { red_child.as_ref() }.color, Color::Red);
        assert_eq!(unsafe { red_p.as_ref() }.color, Color::Red);
        assert_eq!(unsafe { black_g.as_ref() }.color, Color::Black);

        match position {
            NodePosition::Left => {
                self.rotate_right(black_g);
            }
            NodePosition::Right => {
                self.rotate_left(black_g);
            }
        }

        self.color_red(black_g);
        self.color_black(red_p);
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        // println!("REMOVE::: {key}");
        // self.display();
        let removed = self.bs_remove(key);
        // print!("removed:");
        // self.display_node(removed);
        if self.is_nil(removed) {
            return None;
        }

        unsafe {
            // if removed node is root or red, just remove it
            if removed.as_ref().color == Color::Red {
                let removed_box = Box::from_raw(removed.as_ptr());
                let removed_node = *removed_box;
                let value = ManuallyDrop::into_inner(removed_node.value.assume_init());
                self.len -= 1;
                return Some(value);
            }
        }

        let double_black = unsafe {
            let left = removed.as_ref().left;
            let right = removed.as_ref().right;
            if !self.is_nil(left) { left } else { right }
        };
        // print!("double black:");
        // self.display_node(double_black);

        self.remove_fixup(double_black, unsafe { removed.as_ref().parent });

        unsafe {
            let removed_box = Box::from_raw(removed.as_ptr());
            let removed_node = *removed_box;
            let value = ManuallyDrop::into_inner(removed_node.value.assume_init());
            self.len -= 1;
            Some(value)
        }
    }

    fn remove_fixup(&mut self, double_black: NodePtr<K, V>, parent: NodePtr<K, V>) {
        // print!("remove fix up with double black: ");
        // unsafe {
        //     self.display_node(double_black);
        // }
        unsafe {
            if self.is_header(parent) || double_black.as_ref().color == Color::Red {
                self.color_black(double_black);
                return;
            }
        };

        // double black must have sibling
        // we've already excluede the case that removed node is root, so double black now must have parent
        // because removed node is black, if it has no sibling, the black-height of parent will not balance
        // if removed node is right child, and left child is nil (no sibling),
        // the left black-height would be ? + 1 (parent is ?, plus nil 1),
        // while the right black-height would be ? + 1 + x (parent is ?, plus removed node black 1, plus at least one black nil)
        let sibing = self.sibling_of_nil(parent, double_black);
        assert!(!self.is_nil(sibing));

        match unsafe { sibing.as_ref() }.color {
            Color::Black => {
                // case 1: sibling is black
                self.remove_fixup_black_sibling(double_black, parent);
            }
            Color::Red => {
                // case 2: sibling is red, need to transform to case 1
                match self.get_parent_node_position(parent, sibing) {
                    NodePosition::Left => {
                        self.rotate_right(parent);
                    }
                    NodePosition::Right => {
                        self.rotate_left(parent);
                    }
                }
                self.color_black(sibing);
                self.color_red(parent);

                // because sibing is red, the nephew must be both black
                // the nephew will be the new sibing after rotation
                let new_sibing = self.sibling_of_nil(parent, double_black);
                assert_eq!(unsafe { new_sibing.as_ref() }.color, Color::Black);
                self.remove_fixup_black_sibling(double_black, parent);
            }
        }
    }

    fn remove_fixup_black_sibling(&mut self, double_black: NodePtr<K, V>, parent: NodePtr<K, V>) {
        let sibling = self.sibling_of_nil(parent, double_black);

        let (far_nephew, near_nephew) = unsafe {
            let left_nephew = sibling.as_ref().left;
            let right_nephew = sibling.as_ref().right;
            match self.get_parent_node_position(parent, double_black) {
                NodePosition::Left => (right_nephew, left_nephew),
                NodePosition::Right => (left_nephew, right_nephew),
            }
        };

        match unsafe { (far_nephew.as_ref().color, near_nephew.as_ref().color) } {
            (Color::Black, Color::Black) => {
                // case 1-1: if both nephews are black
                //   double-black turns black (black - 1), sibing turn red (black -1), parent becomes double-black (black + 1)
                self.color_red(sibling);
                self.color_black(double_black);
                self.remove_fixup(parent, unsafe { parent.as_ref() }.parent); // here parent.must not be nil
            }
            (Color::Red, _) => {
                self.remove_fixup_far_red_nephew(parent, sibling, double_black, far_nephew)
            }
            (Color::Black, Color::Red) => {
                // case 1-3: if far nephew is black, near nephew is red
                //   - rotate S, let read near nehpew up
                //   - color S red, color red near nephew black
                //   - now it's case 1-2
                match self.get_parent_node_position(sibling, near_nephew) {
                    NodePosition::Left => self.rotate_right(sibling),
                    NodePosition::Right => self.rotate_left(sibling),
                }
                self.color_red(sibling);
                self.color_black(near_nephew);
                self.remove_fixup_far_red_nephew(parent, near_nephew, double_black, sibling);
            }
        }
    }

    fn remove_fixup_far_red_nephew(
        &mut self,
        mut parent: NodePtr<K, V>,
        mut sibling: NodePtr<K, V>,
        double_black: NodePtr<K, V>,
        far_nephew: NodePtr<K, V>,
    ) {
        // case 1-2: if far nephew is red
        //   - rotate P, let S up
        //   - swap the colors of S and P
        //   - color X black (remove the double-black attribute, becase we add a new ancestor black node S)
        //   - color far red nephew black, because we moved one black to X, one black-height of far nephew is missing
        match self.get_parent_node_position(parent, sibling) {
            NodePosition::Left => self.rotate_right(parent),
            NodePosition::Right => self.rotate_left(parent),
        }
        unsafe {
            std::mem::swap(&mut sibling.as_mut().color, &mut parent.as_mut().color);
        };
        self.color_black(double_black);
        self.color_black(far_nephew);
    }

    #[inline]
    fn color_red(&mut self, mut node: NodePtr<K, V>) {
        unsafe {
            node.as_mut().color = Color::Red;
        };
    }

    #[inline]
    fn color_black(&mut self, mut node: NodePtr<K, V>) {
        unsafe {
            node.as_mut().color = Color::Black;
        };
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<K: Key + Debug, V: Value + Debug> RBTree<K, V> {
    /// Prints the tree in a beautiful, human-readable format.
    pub fn display(&self) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                        Red-Black Tree                        ║");
        println!("╠══════════════════════════════════════════════════════════════╣");

        let root = unsafe { self.header.as_ref().right };
        if self.is_nil(root) {
            println!("║                        <EMPTY TREE>                         ║");
            println!("╚═════════════════════════════════════════════════════════════╝");
            return;
        }

        // Count nodes for statistics
        let node_count = self.count_nodes();
        println!("║ Total nodes: {:<47} ║", node_count);
        println!("║ Format: [key:value] (Color) [L/R]                            ║");
        println!("║ Colors: 🔴Red  ⚫Black                                       ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();

        let root_node = unsafe { root.as_ref() };
        let color_symbol = match root_node.color {
            Color::Red => "🔴",
            Color::Black => "⚫",
        };

        println!(
            "{}[{:?}:{:?}] {} [ROOT]",
            color_symbol,
            unsafe { root_node.key() },
            unsafe { root_node.value() },
            color_symbol
        );

        // Display children with proper positioning
        if !self.is_nil(root_node.left) || !self.is_nil(root_node.right) {
            self.display_subtree(root_node.left, root_node.right, "".to_string(), true);
        }

        println!();
    }

    fn display_subtree(
        &self,
        left: NodePtr<K, V>,
        right: NodePtr<K, V>,
        prefix: String,
        is_root_level: bool,
    ) {
        let has_left = !self.is_nil(left);
        let has_right = !self.is_nil(right);

        if has_right {
            let new_prefix = if is_root_level {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            let connector = if has_left { "├── " } else { "└── " };
            let right_node = unsafe { right.as_ref() };
            let color_symbol = match right_node.color {
                Color::Red => "🔴",
                Color::Black => "⚫",
            };

            println!(
                "{}{}{}[{:?}:{:?}] {} [R]",
                prefix,
                connector,
                color_symbol,
                unsafe { right_node.key() },
                unsafe { right_node.value() },
                color_symbol
            );

            if !self.is_nil(right_node.left) || !self.is_nil(right_node.right) {
                self.display_subtree(right_node.left, right_node.right, new_prefix, false);
            }
        }

        if has_left {
            let new_prefix = if is_root_level {
                format!("{}    ", prefix)
            } else {
                format!("{}    ", prefix)
            };

            let left_node = unsafe { left.as_ref() };
            let color_symbol = match left_node.color {
                Color::Red => "🔴",
                Color::Black => "⚫",
            };

            println!(
                "{}└── {}[{:?}:{:?}] {} [L]",
                prefix,
                color_symbol,
                unsafe { left_node.key() },
                unsafe { left_node.value() },
                color_symbol
            );

            if !self.is_nil(left_node.left) || !self.is_nil(left_node.right) {
                self.display_subtree(left_node.left, left_node.right, new_prefix, false);
            }
        }
    }

    /// Alternative compact display format
    pub fn display_compact(&self) {
        print!("RBTree: ");
        let root = unsafe { self.header.as_ref().right };
        if self.is_nil(root) {
            println!("∅");
            return;
        }
        self.display_inorder(root);
        println!();
    }

    fn display_inorder(&self, node: NodePtr<K, V>) {
        if self.is_nil(node) {
            return;
        }

        let node_ref = unsafe { node.as_ref() };
        self.display_inorder(node_ref.left);

        let color_symbol = match node_ref.color {
            Color::Red => "🔴",
            Color::Black => "⚫",
        };
        print!(
            "{}[{:?}:{:?}] ",
            color_symbol,
            unsafe { node_ref.key() },
            unsafe { node_ref.value() }
        );

        self.display_inorder(node_ref.right);
    }

    #[allow(dead_code)]
    fn display_node(&self, node: NodePtr<K, V>) {
        if self.is_nil(node) {
            println!("<nil>");
            return;
        }

        unsafe {
            let color_symbol = match node.as_ref().color {
                Color::Red => "🔴",
                Color::Black => "⚫",
            };
            println!(
                "Node {color_symbol} Key: {:?}, Value:{:?}",
                node.as_ref().key(),
                node.as_ref().value()
            );
        };
    }
}

impl<K: Key + Display + Debug, V: Display + Debug> std::fmt::Display for RBTree<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let root = unsafe { self.header.as_ref().right };
        if self.is_nil(root) {
            write!(f, "RBTree(∅)")
        } else {
            write!(f, "RBTree({} nodes: ", self.count_nodes())?;
            self.fmt_inorder(f, root)?;
            write!(f, ")")
        }
    }
}

impl<K: Key + Display + Debug, V: Display + Debug> RBTree<K, V> {
    fn fmt_inorder(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        node: NodePtr<K, V>,
    ) -> std::fmt::Result {
        if self.is_nil(node) {
            return Ok(());
        }

        let node_ref = unsafe { node.as_ref() };
        self.fmt_inorder(f, node_ref.left)?;

        let color_char = match node_ref.color {
            Color::Red => "R",
            Color::Black => "B",
        };
        write!(
            f,
            "{}:{} ({}) ",
            unsafe { node_ref.key() },
            unsafe { node_ref.value() },
            color_char
        )?;

        self.fmt_inorder(f, node_ref.right)
    }
}

impl<K: Key, V: Value> Drop for RBTree<K, V> {
    fn drop(&mut self) {
        let mut nodes = vec![];
        self.traverse(|node| {
            nodes.push(node);
        });
        for node in nodes {
            unsafe {
                let mut b = Box::from_raw(node.as_ptr()); // don't use * dereference because it requires a copy from heap to stack
                ManuallyDrop::drop(b.key.assume_init_mut()); // just drop on heap
                ManuallyDrop::drop(b.value.assume_init_mut());
                drop(b);
            };
        }

        unsafe {
            drop(Box::from_raw(self.header.as_ptr()));
            drop(Box::from_raw(self.nil.as_ptr()));
        }
    }
}
