use context_iter::ContextIterator;
use std;

#[derive(Clone)]
pub struct Tree<IdType: Copy + Eq, T> {
    id: IdType,
    nodes: Vec<NodeData<T>>,
}

impl<IdType: Copy + Eq, T> Tree<IdType, T> {
    pub fn new(id: IdType) -> Self { Tree { id, nodes: Vec::new() } }

    pub fn create_node(&mut self, data: T) -> Node<IdType> {
        let idx = self.nodes.len();
        self.nodes.push(NodeData::new(data));
        Node { tree_id: self.id, idx }
    }

    pub fn all_nodes(&self) -> AllNodesIter<IdType> {
        AllNodesIter {
            tree_id: self.id,
            cur_idx: 0,
            term_at_idx: self.nodes.len(),
        }
    }

    pub fn first_root_node(&self) -> Option<Node<IdType>> {
        if self.nodes.len() == 0 {
            None
        } else {
            let first_node = Node { tree_id: self.id, idx: 0 };
            Some(first_node.root(self))
        }
    }

    pub fn find_first<Predicate: FnMut(&Node<IdType>) -> bool>(&self, predicate: Predicate) -> Option<Node<IdType>> {
        self.all_nodes().filter(predicate).nth(0)
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct Node<IdType: Copy + Eq> {
    tree_id: IdType,
    idx: usize,
}

impl<IdType: Copy + Eq> Node<IdType> {
    fn validate<T>(&self, t: &Tree<IdType, T>) {
        if self.tree_id != t.id {
            panic!("Attempted to get a value from the wrong tree");
        }
    }
    fn valid_get<'a, T>(&self, t: &'a Tree<IdType, T>) -> &'a NodeData<T> { &t.nodes[self.idx] }
    fn valid_get_mut<'a, T>(&self, t: &'a mut Tree<IdType, T>) -> &'a mut NodeData<T> { &mut t.nodes[self.idx] }
    fn get<'a, T>(&self, t: &'a Tree<IdType, T>) -> &'a NodeData<T> {
        self.validate(t);
        self.valid_get(t)
    }
    fn get_mut<'a, T>(&self, t: &'a mut Tree<IdType, T>) -> &'a mut NodeData<T> {
        self.validate(t);
        self.valid_get_mut(t)
    }

    fn as_idx(&self) -> NodeIdx { NodeIdx { idx: self.idx } }

    pub fn parent<T>(&self, t: &Tree<IdType, T>) -> Option<Node<IdType>> { self.get(t).parent.as_node(self.tree_id) }
    pub fn first_child<T>(&self, t: &Tree<IdType, T>) -> Option<Node<IdType>> { self.get(t).first_child.as_node(self.tree_id) }
    pub fn last_child<T>(&self, t: &Tree<IdType, T>) -> Option<Node<IdType>> { self.get(t).last_child.as_node(self.tree_id) }
    pub fn prev_sibling<T>(&self, t: &Tree<IdType, T>) -> Option<Node<IdType>> { self.get(t).prev_sibling.as_node(self.tree_id) }
    pub fn next_sibling<T>(&self, t: &Tree<IdType, T>) -> Option<Node<IdType>> { self.get(t).next_sibling.as_node(self.tree_id) }
    pub fn value<'a, T>(&self, t: &'a Tree<IdType, T>) -> &'a T { &self.get(t).value }
    pub fn value_mut<'a, T>(&self, t: &'a mut Tree<IdType, T>) -> &'a mut T { &mut self.get_mut(t).value }

    pub fn root<T>(&self, t: &Tree<IdType, T>) -> Node<IdType> {
        let mut result = *self;
        while let Some(p) = result.parent(t) {
            result = p;
        }
        result
    }

    pub fn remove<T>(&self, t: &mut Tree<IdType, T>) {
        let indexes = self.get(t).as_indexes();
        if indexes.parent.is_some() {
            // Fix-up self
            {
                let d = self.valid_get_mut(t);
                d.parent = NodeIdx::none();
                d.prev_sibling = NodeIdx::none();
                d.next_sibling = NodeIdx::none();
            }
            // Fix-up parent
            {
                let d = &mut t.nodes[indexes.parent.idx];
                if d.first_child.is_node(self) {
                    d.first_child = indexes.next_sibling;
                }
                if d.last_child.is_node(self) {
                    d.last_child = indexes.prev_sibling;
                }
            }
            // Fix-up prev sibling
            if indexes.prev_sibling.is_some() {
                let d = &mut t.nodes[indexes.prev_sibling.idx];
                d.next_sibling = indexes.next_sibling;
            }
            // Fix-up next sibling
            if indexes.next_sibling.is_some() {
                let d = &mut t.nodes[indexes.next_sibling.idx];
                d.prev_sibling = indexes.prev_sibling;
            }
        }
    }

    pub fn append_child<T>(&self, t: &mut Tree<IdType, T>, child: Node<IdType>) -> Node<IdType> {
        self.validate(t);
        child.remove(t);

        let last_child = self.valid_get(t).last_child;
        if last_child.is_none() {
            // No existing children
            child.valid_get_mut(t).parent = self.as_idx();
            let d = self.valid_get_mut(t);
            d.first_child = child.as_idx();
            d.last_child = child.as_idx();
        } else {
            // Update current last child
            t.nodes[last_child.idx].next_sibling = child.as_idx();
            // Update the new child
            {
                let d = child.valid_get_mut(t);
                d.prev_sibling = last_child;
                d.parent = self.as_idx();
            }
            // Update self
            self.valid_get_mut(t).last_child = child.as_idx();
        }
        child
    }
    pub fn append_child_value<T>(&self, t: &mut Tree<IdType, T>, child_value: T) -> Node<IdType> {
        let n = t.create_node(child_value);
        self.append_child(t, n);
        n
    }

    pub fn prepend_child<T>(&self, t: &mut Tree<IdType, T>, child: Node<IdType>) -> Node<IdType> {
        self.validate(t);
        child.remove(t);

        let first_child = self.valid_get(t).first_child;
        if first_child.is_none() {
            // No existing children
            child.valid_get_mut(t).parent = self.as_idx();
            let d = self.valid_get_mut(t);
            d.first_child = child.as_idx();
            d.last_child = child.as_idx();
        } else {
            // Update current first child
            t.nodes[first_child.idx].prev_sibling = child.as_idx();
            // Update the new child
            {
                let d = child.valid_get_mut(t);
                d.next_sibling = first_child;
                d.parent = self.as_idx();
            }
            // Update self
            self.valid_get_mut(t).first_child = child.as_idx();
        }
        child
    }
    pub fn prepend_child_value<T>(&self, t: &mut Tree<IdType, T>, child_value: T) -> Node<IdType> {
        let n = t.create_node(child_value);
        self.prepend_child(t, n);
        n
    }

    /// Inserts the specified record immediately after this record. If this record does not have a
    /// parent, then this method will panic.
    pub fn insert_next_sibling<T>(&self, t: &mut Tree<IdType, T>, child: Node<IdType>) -> Node<IdType> {
        let parent = self.parent(t).expect("This node must have a parent node for insert_next_sibling to work.");
        if parent.last_child(t).unwrap() == *self {
            // Current node is the last on in the list, this is the same as calling append_child on the parent
            parent.append_child(t, child)
        } else {
            // In this case, the parent node does not need to be updated.
            self.validate(t);
            child.remove(t);

            let cur_next = self.next_sibling(t).unwrap();

            {
                let d = child.valid_get_mut(t);
                d.parent = parent.as_idx();
                d.prev_sibling = self.as_idx();
                d.next_sibling = cur_next.as_idx();
            }
            self.valid_get_mut(t).next_sibling = child.as_idx();
            cur_next.valid_get_mut(t).prev_sibling = child.as_idx();

            child
        }
    }
    pub fn insert_next_sibling_value<T>(&self, t: &mut Tree<IdType, T>, child_value: T) -> Node<IdType> {
        let n = t.create_node(child_value);
        self.insert_next_sibling(t, n);
        n
    }

    /// Inserts the specified record immediately after this record. If this record does not have a
    /// parent, then this method will panic.
    pub fn insert_prev_sibling<T>(&self, t: &mut Tree<IdType, T>, child: Node<IdType>) -> Node<IdType> {
        let parent = self.parent(t).expect("This node must have a parent node for insert_prev_sibling to work.");
        if parent.first_child(t).unwrap() == *self {
            // Current node is the first on in the list, this is the same as calling prepend_child on the parent
            parent.prepend_child(t, child)
        } else {
            // In this case, the parent node does not need to be updated.
            self.validate(t);
            child.remove(t);

            let cur_next = self.prev_sibling(t).unwrap();

            {
                let d = child.valid_get_mut(t);
                d.parent = parent.as_idx();
                d.prev_sibling = cur_next.as_idx();
                d.next_sibling = self.as_idx();
            }
            self.valid_get_mut(t).prev_sibling = child.as_idx();
            cur_next.valid_get_mut(t).next_sibling = child.as_idx();

            child
        }
    }
    pub fn insert_prev_sibling_value<T>(&self, t: &mut Tree<IdType, T>, child_value: T) -> Node<IdType> {
        let n = t.create_node(child_value);
        self.insert_prev_sibling(t, n);
        n
    }

    /// Removes all child nodes from this node
    pub fn remove_children<T>(&self, t: &mut Tree<IdType, T>) {
        while let Some(c) = self.first_child(t) {
            c.remove(t);
        }
    }

    /// Returns a standard iterator for all children of this node. Holds a reference to the tree
    /// for the duration of the iterator.
    pub fn children<'a, T>(&self, t: &'a Tree<IdType, T>) -> ContextFreeSiblingIter<'a, IdType, T> {
        let next = self.first_child(t);
        ContextFreeSiblingIter { next, tree: t }
    }

    /// Returns a context iterator (requiring the tree reference) for all children of this node.
    /// This function can be used in situations where the tree (or its data) needs to be altered
    /// while iterating over the list of children.
    ///
    /// Internally, this calls next_sibling for each child record. Removing upcoming siblings or
    /// altering their next_sibling value will cause the iterator to go off-track. Memory safety is
    /// still upheld, but the results will not be as expected or may not terminate.
    ///
    /// Be sure to either import trees::ContextIterator or use the next_value method.
    pub fn children_mut<T>(&self, t: &Tree<IdType, T>) -> SiblingIter<IdType, T> { SiblingIter::new(self.first_child(t)) }

    /// Returns a standard iterator starting with the current node, expanding to all nodes
    /// underneath this node. The iterator returns a tuple containing the depth (starting with
    /// zero) and the current node.
    ///
    /// This is a depth-first search using pre-order.
    pub fn depth_first_search<'a, T>(&self, t: &'a Tree<IdType, T>) -> ContextFreeDepthFirstIter<'a, IdType, T> {
        let iter = self.depth_first_search_mut();
        ContextFreeDepthFirstIter { iter, context: t }
    }

    pub fn depth_first_search_mut<T>(&self) -> DepthFirstIter<IdType, T> { DepthFirstIter::new(*self) }

    pub fn find_first<T, Predicate: FnMut(&(usize, Node<IdType>)) -> bool>(&self, t: &Tree<IdType, T>, predicate: Predicate) -> Option<(usize, Node<IdType>)> {
        self.depth_first_search(t).filter(predicate).nth(0)
    }

    //pub fn breadth_first_search
}

struct NodeIndexes {
    parent: NodeIdx,
    prev_sibling: NodeIdx,
    next_sibling: NodeIdx,
}

#[derive(Clone)]
struct NodeData<T> {
    value: T,
    parent: NodeIdx,
    first_child: NodeIdx,
    last_child: NodeIdx,
    prev_sibling: NodeIdx,
    next_sibling: NodeIdx,
}
impl<T> NodeData<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            parent: NodeIdx::none(),
            first_child: NodeIdx::none(),
            last_child: NodeIdx::none(),
            prev_sibling: NodeIdx::none(),
            next_sibling: NodeIdx::none(),
        }
    }
    pub fn as_indexes(&self) -> NodeIndexes {
        NodeIndexes {
            parent: self.parent,
            prev_sibling: self.prev_sibling,
            next_sibling: self.next_sibling,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
struct NodeIdx {
    idx: usize,
}
impl NodeIdx {
    pub fn none() -> Self { Self { idx: std::usize::MAX } }
    pub fn is_none(&self) -> bool { self.idx == std::usize::MAX }
    pub fn is_some(&self) -> bool { !self.is_none() }
    pub fn is_node<IdType: Copy + Eq>(&self, n: &Node<IdType>) -> bool { self.idx == n.idx }
    pub fn as_node<IdType: Copy + Eq>(&self, tree_id: IdType) -> Option<Node<IdType>> {
        if self.is_none() {
            None
        } else {
            Some(Node {
                tree_id: tree_id,
                idx: self.idx,
            })
        }
    }
}

pub struct DepthFirstIter<IdType: Copy + Eq, T> {
    next: Option<Node<IdType>>,
    parents: Vec<Node<IdType>>,
    _marker: std::marker::PhantomData<T>,
}
impl<IdType: Copy + Eq, T> DepthFirstIter<IdType, T> {
    fn new(root: Node<IdType>) -> Self {
        Self {
            next: Some(root),
            parents: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn next_value(&mut self, t: &Tree<IdType, T>) -> Option<(usize, Node<IdType>)> {
        if let Some(n) = self.next {
            let depth = self.parents.len();
            let result = Some((depth, n));
            if let Some(child) = n.first_child(t) {
                // Node has children, start there next time
                self.parents.push(n);
                self.next = Some(child);
            } else if let Some(sib) = n.next_sibling(t) {
                // Node has a sibling, start there next time
                self.next = Some(sib);
            } else {
                // No more children/siblings to process, find a parent's sibling to process
                let mut parent = self.parents.pop();
                while let Some(p) = parent {
                    if let Some(s) = p.next_sibling(t) {
                        self.next = Some(s);
                        return result;
                    }
                    parent = self.parents.pop();
                }
                self.next = None;
            }
            result
        } else {
            None
        }
    }

    pub fn discard_child_results(&mut self, t: &Tree<IdType, T>, cur_depth: usize) {
        if self.parents.len() > cur_depth {
            let mut parent = self.parents.pop();
            while self.parents.len() > cur_depth {
                parent = self.parents.pop();
            }

            // Note: Copied from the next_value function
            while let Some(p) = parent {
                if let Some(s) = p.next_sibling(t) {
                    self.next = Some(s);
                    return;
                }
                parent = self.parents.pop();
            }
            self.next = None;
        }
    }
}
impl<IdType: Copy + Eq, T> ContextIterator<Tree<IdType, T>> for DepthFirstIter<IdType, T> {
    type Item = (usize, Node<IdType>);

    fn next(&mut self, t: &Tree<IdType, T>) -> Option<Self::Item> { self.next_value(t) }
}

pub struct ContextFreeDepthFirstIter<'a, IdType: Copy + Eq + 'a, T: 'a> {
    iter: DepthFirstIter<IdType, T>,
    context: &'a Tree<IdType, T>,
}
impl<'a, IdType: Copy + Eq + 'a, T: 'a> ContextFreeDepthFirstIter<'a, IdType, T> {
    pub fn discard_child_results(&mut self, cur_depth: usize) { self.iter.discard_child_results(self.context, cur_depth) }
}
impl<'a, IdType: Copy + Eq + 'a, T: 'a> std::iter::Iterator for ContextFreeDepthFirstIter<'a, IdType, T> {
    type Item = (usize, Node<IdType>);

    fn next(&mut self) -> Option<Self::Item> { self.iter.next_value(self.context) }
}

pub struct AllNodesIter<IdType: Copy + Eq> {
    tree_id: IdType,
    cur_idx: usize,
    term_at_idx: usize,
}
impl<IdType: Copy + Eq> std::iter::Iterator for AllNodesIter<IdType> {
    type Item = Node<IdType>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur_idx == self.term_at_idx {
            None
        } else {
            let n = Node {
                tree_id: self.tree_id.clone(),
                idx: self.cur_idx,
            };
            self.cur_idx += 1;
            Some(n)
        }
    }
}

pub struct SiblingIter<IdType: Copy + Eq, T> {
    next: Option<Node<IdType>>,
    _marker: std::marker::PhantomData<T>,
}
impl<IdType: Copy + Eq, T> SiblingIter<IdType, T> {
    fn new(next: Option<Node<IdType>>) -> Self {
        Self {
            next,
            _marker: std::marker::PhantomData,
        }
    }
    pub fn next_value(&mut self, t: &Tree<IdType, T>) -> Option<Node<IdType>> {
        if let Some(n) = self.next {
            self.next = n.next_sibling(t);
            Some(n)
        } else {
            None
        }
    }
}
impl<IdType: Copy + Eq, T> ContextIterator<Tree<IdType, T>> for SiblingIter<IdType, T> {
    type Item = Node<IdType>;

    fn next(&mut self, t: &Tree<IdType, T>) -> Option<Self::Item> { self.next_value(t) }
}

pub struct ContextFreeSiblingIter<'a, IdType: Copy + Eq + 'a, T: 'a> {
    next: Option<Node<IdType>>,
    tree: &'a Tree<IdType, T>,
}
impl<'a, IdType: Copy + Eq + 'a, T: 'a> std::iter::Iterator for ContextFreeSiblingIter<'a, IdType, T> {
    type Item = Node<IdType>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(n) = self.next {
            let next = n.next_sibling(self.tree);
            self.next = next;
            Some(n)
        } else {
            None
        }
    }
}
