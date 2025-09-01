use rb_tree::SimpleBST;

fn main() {
    let mut bst = SimpleBST::new();

    // Test insertions
    println!("=== Testing Binary Search Tree ===");
    println!("Inserting values: 5, 3, 7, 2, 4, 6, 8");
    
    bst.insert(5, "five");
    bst.insert(3, "three");
    bst.insert(7, "seven");
    bst.insert(2, "two");
    bst.insert(4, "four");
    bst.insert(6, "six");
    bst.insert(8, "eight");

    println!("Tree length: {}", bst.len());
    println!("Is empty: {}", bst.is_empty());

    // Test search operations
    println!("\n=== Testing Search Operations ===");
    println!("Search for key 5: {:?}", bst.get(&5));
    println!("Search for key 9: {:?}", bst.get(&9));
    println!("Search for key 3: {:?}", bst.get(&3));

    // Test mutable search
    if let Some(value) = bst.get_mut(&3) {
        *value = "THREE";
        println!("Updated key 3 to: {}", value);
    }

    // Test traversal (in-order)
    println!("\n=== In-order Traversal ===");
    print!("Keys in order: ");
    bst.traverse_kv(|key, value| {
        print!("{}:{} ", key, value);
    });
    println!();

    // Test removal
    println!("\n=== Testing Removal ===");
    println!("Removing key 3: {:?}", bst.remove(&3));
    println!("Tree length after removal: {}", bst.len());
    println!("Search for removed key 3: {:?}", bst.get(&3));

    println!("Removing key 5 (root): {:?}", bst.remove(&5));
    println!("Tree length after removing root: {}", bst.len());

    // Final traversal
    println!("\n=== Final In-order Traversal ===");
    print!("Remaining keys: ");
    bst.traverse_kv(|key, value| {
        print!("{}:{} ", key, value);
    });
    println!();

    println!("\n=== Test completed successfully! ===");
}