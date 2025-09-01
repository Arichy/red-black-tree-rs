use std::{
    fmt::{Debug, Display},
    mem::MaybeUninit,
    panic,
    ptr::NonNull,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Color {
    Red,
    Black,
}

pub trait Key: PartialEq + PartialOrd + Display + Debug {}
impl<T> Key for T where T: PartialEq + PartialOrd + Display + Debug {}

pub trait Value: Debug + Display {}
impl<T> Value for T where T: Debug + Display {}

pub(crate) type NodePtr<K, V> = NonNull<RBNode<K, V>>;

#[derive(Debug)]
pub struct RBNode<K: Key, V: Value> {
    pub(crate) key: MaybeUninit<K>,
    pub(crate) value: MaybeUninit<V>,
    pub(crate) color: Color,
    pub(crate) left: NodePtr<K, V>,
    pub(crate) right: NodePtr<K, V>,
    pub(crate) parent: NodePtr<K, V>,
}

// impl<K: Key, V: Value> RBNode<K, V> {
//     pub fn drop(ptr: NodePtr<K, V>) {
//         unsafe {
//             let _ = Box::from_raw(ptr.as_ptr());
//         };
//     }
// }
