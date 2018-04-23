# Trees Library #

Single-threaded: `trees::rctree::RcNode`
- Easy-to-use (does not require Tree reference)
- Single thread only

Insert-only Arena: `trees::arena::{Tree, Node}`
- If a node is orphaned, it stays in memory until entire tree is dropped

Delete on Remove: `trees::withdelete::{Tree, Node}`
- If a node is inaccessible from the root node, it is immediately flagged as deleted
- Space used by deleted nodes can be reused

Immutable, thread-safe: `trees::immutable::Node`
- Easy-to-use (does not require Tree reference)
- Read-only use
