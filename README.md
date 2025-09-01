# Red-Black Tree Implementation in Rust 🌳

[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Educational](https://img.shields.io/badge/purpose-educational%20%26%20research-green.svg)]()

**🎓 Educational & Research Project**

A comprehensive implementation of a Red-Black Tree data structure in Rust, designed for **learning, research, and algorithm study purposes**. This project demonstrates the principles of self-balancing binary search trees with detailed implementation and extensive testing.

> ⚠️ **Note**: This is an educational/research project focused on understanding Red-Black Tree algorithms. While functional and well-tested, it is **not intended for production use**. For production applications, consider using Rust's standard library `BTreeMap` or other mature libraries.

## Getting Started 📦

### For Learning and Research

Clone this repository to explore the Red-Black Tree implementation:

```bash
git clone <repository-url>
cd rb_tree
cargo build
```

> 📚 **Recommended**: Study the source code in `src/` to understand the Red-Black Tree algorithms and implementation details.

## Quick Start 🚀

```rust
use rb_tree::RBTree;

fn main() {
    let mut tree = RBTree::new();

    // Insert key-value pairs
    tree.insert(5, "five");
    tree.insert(3, "three");
    tree.insert(7, "seven");
    tree.insert(1, "one");

    // Search for values
    if let Some(value) = tree.get(&3) {
        println!("Found: {}", value); // Found: three
    }

    // Remove elements
    tree.remove(&3);

    // Iterate over the tree (in-order traversal)
    for (key, value) in &tree {
        println!("{}: {}", key, value);
    }
}
```

## API Reference 📚

### Core Operations

- `RBTree::new()` - Create a new empty tree
- `insert(key, value)` - Insert a key-value pair, returns old value if key existed
- `get(key)` - Search for a value by key, returns `Option<&V>`
- `get_mut(key)` - Get mutable reference to value by key
- `remove(key)` - Remove a key-value pair, returns the removed value
- `len()` - Get the number of elements in the tree

### Iteration

- `iter()` - Create an iterator over key-value pairs
- `into_iter()` - Create a consuming iterator

## Examples 💡

Check out the [examples](examples/) directory for detailed usage examples:

- [`basic.rs`](examples/basic.rs) - Basic operations with performance timing
- [`iter.rs`](examples/iter.rs) - Iterator usage examples
- [`binary_search_tree.rs`](examples/binary_search_tree.rs) - Comparison with simple BST

Run examples with:

```bash
cargo run --example basic
cargo run --example iter
```

## Benchmarks 📊

Our Red-Black Tree implementation is benchmarked against Rust's standard `BTreeMap` and a simple Binary Search Tree (BST) implementation. Here are the performance characteristics:

### Insert Operations

Performance comparison for inserting elements into different tree structures:

| Data Structure | Random Insertions (5000 elements) | Sequential Insertions (5000 elements) |
| -------------- | --------------------------------- | ------------------------------------- |
| **RBTree**     | ~392 μs ⚡                        | ~322 μs ⚡                            |
| **BTreeMap**   | ~142 μs 🏆                        | ~155 μs 🏆                            |
| **Simple BST** | ~415 μs                           | ~35.3 ms ⚠️ (degenerate case)         |

**Key Insights:**

- ✅ RBTree maintains consistent O(log n) performance regardless of input pattern
- ✅ BTreeMap shows slightly better performance due to cache-friendly B-tree structure
- ⚠️ Simple BST degrades to O(n) with sequential input, becoming ~100x slower

### Search Operations

Search performance on a tree with 10,000 elements:

| Data Structure | Random Search Time |
| -------------- | ------------------ |
| **RBTree**     | ~8 ns ⚡           |
| **BTreeMap**   | ~8 ns 🏆           |
| **Simple BST** | ~13 ns             |

### Remove Operations

Removal performance on trees with 10,000 elements:

| Data Structure | Random Removal Time |
| -------------- | ------------------- |
| **RBTree**     | ~432 μs ⚡          |
| **BTreeMap**   | ~68 μs 🏆           |
| **Simple BST** | ~450 μs             |

### Benchmark Summary (Educational Insights)

**Educational Value of RBTree:**

- 🎯 **Algorithm Study**: Understanding self-balancing tree mechanics
- 📊 **Performance Analysis**: Comparing theoretical vs. practical performance
- 🔍 **Implementation Details**: Learning low-level data structure implementation
- 🧪 **Testing Methodologies**: Exploring property-based and differential testing

**For Production Use, Consider:**

- 🏭 **BTreeMap**: Rust's standard library implementation with production-grade optimizations
- 🚀 **Third-party crates**: Mature libraries like `indexmap`, `im`, or specialized collections
- 💡 **This project serves as**: A reference implementation for understanding the algorithms

### Running Benchmarks

To run the benchmarks yourself:

```bash
cargo bench
```

Benchmark reports are generated in `target/criterion/` with detailed HTML reports.

## Testing 🧪

The project includes comprehensive testing:

- **Unit tests**: Core functionality testing
- **Integration tests**: End-to-end behavior validation
- **Property-based tests**: Automated testing with random inputs using `proptest`
- **Differential testing**: Comparison with reference implementations

Run all tests:

```bash
cargo test
```

Run property-based tests with more iterations:

```bash
cargo test --release prop_test
```

## Architecture 🏗️

The implementation consists of several key modules:

- **`node.rs`**: Red-Black Tree node structure and memory management
- **`binary_tree.rs`**: Basic binary tree operations and rotations
- **`binary_search_tree/`**: Binary search tree implementation and validation
- **`iter.rs`**: Iterator implementations for tree traversal
- **`validate.rs`**: Red-Black Tree property validation
