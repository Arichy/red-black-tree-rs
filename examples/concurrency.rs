use std::{
    sync::{LazyLock, RwLock},
    thread,
};

use rb_tree::RBTree;

static RB_TREE: LazyLock<RwLock<RBTree<String, i32>>> =
    LazyLock::new(|| RwLock::new(RBTree::new()));

fn main() {
    let t1 = thread::spawn(|| {
        for i in 0..=10 {
            let mut tree = RB_TREE.write().unwrap();
            tree.insert(format!("key{}", i), i);
        }
    });

    let t2 = thread::spawn(|| {
        for i in 11..=20 {
            let mut tree = RB_TREE.write().unwrap();
            tree.insert(format!("key{}", i), i);
        }
    });

    let t3 = thread::spawn(|| {
        for i in 21..=30 {
            let mut tree = RB_TREE.write().unwrap();
            tree.insert(format!("key{}", i), i);
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();
    t3.join().unwrap();

    let read_thread_1 = thread::spawn(|| {
        for i in 0..=10 {
            let tree = RB_TREE.read().unwrap();
            println!("{:?}", tree.get(&format!("key{}", i)));
        }
    });

    let read_thread_2 = thread::spawn(|| {
        for i in 11..=20 {
            let tree = RB_TREE.read().unwrap();
            println!("{:?}", tree.get(&format!("key{}", i)));
        }
    });

    let read_thread_3 = thread::spawn(|| {
        for i in 21..=30 {
            let tree = RB_TREE.read().unwrap();
            println!("{:?}", tree.get(&format!("key{}", i)));
        }
    });

    read_thread_1.join().unwrap();
    read_thread_2.join().unwrap();
    read_thread_3.join().unwrap();
}
