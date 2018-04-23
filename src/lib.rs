pub mod arena;
mod context_iter;
pub mod rctree;

/*
TODO Add the following:
immutable - uses Arc for data storage (thread-safe, does not require context)
withdelete - adds deleted flag, useful if the tree will have a lot of removals and additions
             include a created id to ensure deleted nodes cannot be referenced again?
*/

pub use context_iter::ContextIterator;
