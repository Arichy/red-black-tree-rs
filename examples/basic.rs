use std::{collections::HashSet, time::Instant};

use rb_tree::RBTree;

fn main() {
    let mut tree = RBTree::new();

    let mut unique_key = HashSet::new();
    for _ in 0..4000 {
        let rand_i32 = rand::random::<i32>();
        let now = Instant::now();
        tree.insert(rand_i32, rand_i32);
        println!("Insert duration for {rand_i32}: {:?}", now.elapsed());
        unique_key.insert(rand_i32);
    }

    if let Err(e) = tree.validate() {
        panic!("ERR: {e}");
    }

    for key in &unique_key {
        let now = Instant::now();
        if let Some(v) = tree.get(key) {
            if v != key {
                panic!()
            }
        } else {
            panic!("KEY {key} not found")
        }
        println!("Search duration for key: {key} {:?}", now.elapsed());
    }

    for key in &unique_key {
        let now = Instant::now();
        println!("Remove duration for {key}: {:?}", now.elapsed());
        tree.remove(key);

        let now = Instant::now();
        if let Err(e) = tree.validate() {
            panic!("ERR: {e}");
        }
        println!("Validate duration for {key}: {:?}", now.elapsed());
    }
}
