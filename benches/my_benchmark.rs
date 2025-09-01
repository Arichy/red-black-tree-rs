use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::{Rng, seq::SliceRandom};
use std::{collections::BTreeMap, hint::black_box};

use rb_tree::{RBTree, SimpleBST};

// fn criterion_benchmark(c: &mut Criterion) {
//     c.bench_function("fib 20", |b| b.iter(|| fibonacci(black_box(20))));
// }

fn bench_inserts(c: &mut Criterion) {
    let mut group = c.benchmark_group("Inserts");

    for size in [100, 500, 1000, 5000].iter() {
        let mut rng = rand::rng();
        let mut keys: Vec<u32> = (0..*size).collect();
        keys.shuffle(&mut rng);

        group.bench_with_input(
            BenchmarkId::new("RBTree (Random)", size),
            &keys,
            |b, keys| {
                b.iter(|| {
                    let mut tree = RBTree::new();
                    for &key in keys {
                        tree.insert(key, key);
                    }
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("BST (Random)", size), &keys, |b, keys| {
            b.iter(|| {
                let mut tree = SimpleBST::new();
                for &key in keys {
                    tree.insert(key, key);
                }
            });
        });

        group.bench_with_input(
            BenchmarkId::new("BTreeMap (Random)", size),
            &keys,
            |b, keys| {
                b.iter(|| {
                    let mut tree = BTreeMap::new();
                    for &key in keys {
                        tree.insert(key, key);
                    }
                });
            },
        );

        let sorted_keys: Vec<u32> = (0..*size).collect();

        group.bench_with_input(
            BenchmarkId::new("RBTree (Sequential)", size),
            &sorted_keys,
            |b, keys| {
                b.iter(|| {
                    let mut tree = RBTree::new();
                    for &key in keys {
                        tree.insert(key, key);
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("BST (Sequential)", size),
            &sorted_keys,
            |b, keys| {
                b.iter(|| {
                    let mut tree = SimpleBST::new();
                    for &key in keys {
                        tree.insert(key, key);
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("BTreeMap (Sequential)", size),
            &sorted_keys,
            |b, keys| {
                b.iter(|| {
                    let mut tree = BTreeMap::new();
                    for &key in keys {
                        tree.insert(key, key);
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_searches(c: &mut Criterion) {
    let mut group = c.benchmark_group("Searches");
    let size = 10_000;
    let mut rng = rand::rng();
    let mut keys: Vec<u32> = (0..size).collect();
    keys.shuffle(&mut rng);

    let mut rb_tree = RBTree::new();
    let mut bst_tree = SimpleBST::new();
    let mut btree_map = BTreeMap::new();
    for &key in keys.iter() {
        rb_tree.insert(key, key);
        bst_tree.insert(key, key);
        btree_map.insert(key, key);
    }

    let key_to_find = keys[rng.random_range(0..size) as usize];

    group.bench_function("RBTree (Random)", |b| {
        b.iter(|| {
            black_box(rb_tree.get(&key_to_find));
        })
    });

    group.bench_function("BST (Random)", |b| {
        b.iter(|| {
            black_box(bst_tree.get(&key_to_find));
        })
    });

    group.bench_function("BTreeMap (Random)", |b| {
        b.iter(|| {
            black_box(btree_map.get(&key_to_find));
        })
    });

    group.finish();
}

fn bench_removes(c: &mut Criterion) {
    let mut group = c.benchmark_group("Removes");
    let size = 10_000;

    let mut rng = rand::rng();
    let mut keys: Vec<u32> = (0..size).collect();
    keys.shuffle(&mut rng);

    group.bench_function("RBTree (Random)", |b| {
        b.iter_batched(
            || {
                let mut tree = RBTree::new();
                for &key in &keys {
                    tree.insert(key, key);
                }
                let key_to_remove = keys[rng.random_range(0..keys.len())];
                (tree, key_to_remove)
            },
            |(mut tree, key_to_remove)| {
                tree.remove(&key_to_remove);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("BST (Random)", |b| {
        b.iter_batched(
            || {
                let mut tree = SimpleBST::new();
                for &key in &keys {
                    tree.insert(key, key);
                }
                let key_to_remove = keys[rng.random_range(0..keys.len())];
                (tree, key_to_remove)
            },
            |(mut tree, key_to_remove)| {
                tree.remove(&key_to_remove);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("BTreeMap (Random)", |b| {
        b.iter_batched(
            || {
                let mut tree = BTreeMap::new();
                for &key in &keys {
                    tree.insert(key, key);
                }
                let key_to_remove = keys[rng.random_range(0..keys.len())];
                (tree, key_to_remove)
            },
            |(mut tree, key_to_remove)| {
                tree.remove(&key_to_remove);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(benches, bench_inserts, bench_searches, bench_removes);
criterion_main!(benches);
