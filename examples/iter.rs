use rb_tree::RBTree;

fn main() {
    let mut tree = RBTree::new();
    tree.insert(1, 1);
    tree.insert(2, 2);

    tree.display();

    for item in tree {
        println!("{:?}", item);
    }
}
