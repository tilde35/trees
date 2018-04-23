use std;
use std::cell::Cell;
use std::rc::{Rc, Weak};

struct WeakLink<Value>(Cell<Option<Weak<NodeData<Value>>>>);
impl<Value> WeakLink<Value> {
    pub fn new() -> Self { WeakLink(Cell::new(None)) }
    pub fn is_some(&self) -> bool { unsafe { (*self.0.as_ptr()).is_some() } }
    pub fn get(&self) -> Option<RcNode<Value>> { unsafe { (*self.0.as_ptr()).clone().and_then(|d| d.upgrade()).map(|d| RcNode(d)) } }
    pub fn set(&self, val: Option<&RcNode<Value>>) { self.0.set(val.map(|n| Rc::downgrade(&n.0))) }
}

struct StrongLink<Value>(Cell<Option<Rc<NodeData<Value>>>>);
impl<Value> StrongLink<Value> {
    pub fn new() -> Self { StrongLink(Cell::new(None)) }
    pub fn get(&self) -> Option<RcNode<Value>> { unsafe { (*self.0.as_ptr()).clone().map(|d| RcNode(d)) } }
    pub fn set(&self, val: Option<&RcNode<Value>>) { self.0.set(val.map(|n| n.0.clone())) }
}

struct NodeData<Value> {
    parent: WeakLink<Value>,
    next_sibling: StrongLink<Value>,
    prev_sibling: WeakLink<Value>,
    first_child: StrongLink<Value>,
    last_child: StrongLink<Value>,
    value: Value,
}

pub struct RcNode<Value>(Rc<NodeData<Value>>);
impl<Value> RcNode<Value> {
    pub fn new(value: Value) -> Self {
        let d = NodeData {
            parent: WeakLink::new(),
            next_sibling: StrongLink::new(),
            prev_sibling: WeakLink::new(),
            first_child: StrongLink::new(),
            last_child: StrongLink::new(),
            value: value,
        };
        RcNode(Rc::new(d))
    }

    fn ptr_eq(&self, other: &Self) -> bool { Rc::ptr_eq(&self.0, &other.0) }

    pub fn parent(&self) -> Option<RcNode<Value>> { self.0.parent.get() }
    pub fn next_sibling(&self) -> Option<RcNode<Value>> { self.0.next_sibling.get() }
    pub fn prev_sibling(&self) -> Option<RcNode<Value>> { self.0.prev_sibling.get() }
    pub fn first_child(&self) -> Option<RcNode<Value>> { self.0.first_child.get() }
    pub fn last_child(&self) -> Option<RcNode<Value>> { self.0.last_child.get() }
    pub fn value(&self) -> &Value { &self.0.value }
    pub fn children(&self) -> RcNodeSiblingIter<Value> { RcNodeSiblingIter { next: self.first_child() } }

    /// Add the specified child to this node after the last existing child (if any).
    /// If the node already exists in a differnt tree/location, then it is removed from the old location and added to this one.
    pub fn append_child(&self, child: &Self) {
        // Note: By checking parent.is_some(), it ensures that remove is called even if parent was deleted
        if child.0.parent.is_some() {
            child.remove();
        }
        self.unchecked_append_child(child)
    }
    pub fn append_child_value(&self, value: Value) -> RcNode<Value> {
        let child = RcNode::new(value);
        self.unchecked_append_child(&child);
        child
    }
    fn unchecked_append_child(&self, child: &Self) {
        let parent = self;
        child.0.parent.set(Some(parent));
        if let Some(lc) = parent.0.last_child.get() {
            parent.0.last_child.set(Some(child));
            child.0.prev_sibling.set(Some(&lc));
            lc.0.next_sibling.set(Some(child));
        } else {
            parent.0.first_child.set(Some(child));
            parent.0.last_child.set(Some(child));
        }
    }

    /// Removes all child nodes from this node
    pub fn remove_children(&self) {
        while let Some(c) = self.first_child() {
            c.remove();
        }
    }

    /// Removes this node from its parent tree
    pub fn remove(&self) {
        let parent = self.parent();
        let prev = self.prev_sibling();
        let next = self.next_sibling();
        let prev = prev.as_ref();
        let next = next.as_ref();

        if let Some(p) = parent {
            // Note: self record is a child, so we are okay to call child().unwrap() here
            let is_first = p.first_child().unwrap().ptr_eq(self);
            let is_last = p.last_child().unwrap().ptr_eq(self);
            if is_first {
                p.0.first_child.set(next);
            }
            if is_last {
                p.0.last_child.set(prev);
            }
        }

        if let Some(s) = prev {
            s.0.next_sibling.set(next);
        }

        if let Some(s) = next {
            s.0.prev_sibling.set(prev);
        }

        self.0.parent.set(None);
        self.0.prev_sibling.set(None);
        self.0.next_sibling.set(None);
    }
}
impl<Value> std::clone::Clone for RcNode<Value> {
    fn clone(&self) -> Self { RcNode(self.0.clone()) }
}
impl<Value: std::fmt::Debug> std::fmt::Debug for RcNode<Value> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        self.value().fmt(f)?;
        if self.first_child().is_some() {
            f.write_str(" [")?;
            for c in self.children() {
                c.fmt(f)?;
            }
            f.write_str("]")?;
        }
        Ok(())
    }
}

pub struct RcNodeSiblingIter<Value> {
    next: Option<RcNode<Value>>,
}
impl<Value> std::iter::Iterator for RcNodeSiblingIter<Value> {
    type Item = RcNode<Value>;

    fn next(&mut self) -> Option<RcNode<Value>> {
        if let Some(n) = self.next.clone() {
            self.next = n.next_sibling();
            Some(n)
        } else {
            None
        }
    }
}
