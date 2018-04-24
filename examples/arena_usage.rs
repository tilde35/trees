extern crate trees;

use trees::arena::Tree;

fn print_tree(t: &Tree<(), String>, title: &str) {
    println!("{}", title);
    if let Some(root) = t.first_root_node() {
        for (depth, node) in root.depth_first_search(t) {
            for _ in 0..depth {
                print!("  ");
            }
            println!("- {}", node.value(t));
        }
    }
    println!("");
}

fn main() {
    // Create the backing tree structure
    let mut tree: Tree<(), String> = Tree::new(());

    // Build the tree
    let root = {
        let t = &mut tree;
        let root = t.create_node("Root".into());
        let a = root.append_child_value(t, "Parent A".into());
        a.append_child_value(t, "Child A1".into());
        a.append_child_value(t, "Child A2".into());
        a.append_child_value(t, "Child A3".into());
        let b = root.append_child_value(t, "Parent B".into());
        b.append_child_value(t, "Child B1".into());
        let c = root.append_child_value(t, "Parent C".into());
        c.append_child_value(t, "Child C1".into());
        c.append_child_value(t, "Child C2".into());
        c.append_child_value(t, "Child C3".into());
        c.append_child_value(t, "Child C4".into());

        print_tree(t, "=== Tree Structure ===");

        root
    };

    // Remove nodes
    {
        let mut tree = tree.clone();
        let t = &mut tree;

        let c = t.find_first(|n| n.value(t) == "Parent C").unwrap();
        c.remove(t);

        print_tree(t, "=== Removed Parent C ===");
    }

    // Remove children
    {
        let mut tree = tree.clone();
        let t = &mut tree;

        let a = t.find_first(|n| n.value(t) == "Parent A").unwrap();
        a.remove_children(t);

        print_tree(t, "=== Removed Parent A's Child Nodes ===");
    }

    // Update children while iterating
    {
        let mut tree = tree.clone();
        let t = &mut tree;

        let a = t.find_first(|n| n.value(t) == "Parent A").unwrap();

        let mut iter = a.children_mut(t);
        while let Some(child) = iter.next_value(t) {
            child.value_mut(t).push_str(" - Updated");
        }

        print_tree(t, "=== Update Parent A's Children While Iterating (children_mut) ===");
    }

    // Children using Standard (Read-Only) Iterator
    {
        let t = &tree;

        let a = root.first_child(t).unwrap();

        let names: Vec<_> = a.children(t).map(|n| n.value(t)).collect();

        println!("=== Names of Parent A's Children using Standard Iterator ===");
        println!("  {:?}", &names);
        println!("");
    }

    // Root method
    {
        let mut tree = tree.clone();
        let t = &mut tree;

        let a = root.first_child(t).unwrap();
        let a1 = a.first_child(t).unwrap();

        println!("=== Root Values ===");
        println!("Root node for {:?} => {:?}", root.value(t), root.root(t).value(t));
        println!("Root node for {:?} => {:?}", a.value(t), a.root(t).value(t));
        println!("Root node for {:?} => {:?}", a1.value(t), a1.root(t).value(t));

        a.remove(t);
        println!("After removing {:?}", a.value(t));
        println!("Root node for {:?} => {:?}", root.value(t), root.root(t).value(t));
        println!("Root node for {:?} => {:?}", a.value(t), a.root(t).value(t));
        println!("Root node for {:?} => {:?}", a1.value(t), a1.root(t).value(t));

        println!("");
    }

    // Depth-first search with discard
    {
        let t = &tree;

        println!("=== Depth-first Search with Discard ===");

        let mut iter = root.depth_first_search(t);
        while let Some((depth, node)) = iter.next() {
            for _ in 0..depth {
                print!("  ");
            }
            println!("- {}", node.value(t));

            if node.children(t).count() >= 3 && depth >= 1 {
                // Discard the child records
                iter.discard_child_results(depth);

                for _ in 0..(depth + 1) {
                    print!("  ");
                }
                println!("+ ({} child records omitted)", node.children(t).count());
            }
        }
    }

    // TODO Add prepend_child* functions
    // TODO Add insert_next_sibling* functions
    // TODO Add insert_prev_sibling* functions
}
