use std::{
    fmt::{Debug, Display},
    mem::{ManuallyDrop, MaybeUninit},
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
    pub(crate) key: MaybeUninit<ManuallyDrop<K>>,
    pub(crate) value: MaybeUninit<ManuallyDrop<V>>,
    pub(crate) color: Color,
    pub(crate) left: NodePtr<K, V>,
    pub(crate) right: NodePtr<K, V>,
    pub(crate) parent: NodePtr<K, V>,
}

impl<K: Key, V: Value> RBNode<K, V> {
    pub(crate) unsafe fn key(&self) -> &K {
        self.key.assume_init_ref()
    }

    pub(crate) unsafe fn key_mut(&mut self) -> &mut K {
        self.key.assume_init_mut()
    }

    pub(crate) unsafe fn value(&self) -> &V {
        self.value.assume_init_ref()
    }

    pub(crate) unsafe fn value_mut(&mut self) -> &mut V {
        self.value.assume_init_mut()
    }
}
