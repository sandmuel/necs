use crate::NodeKey;
use crate::storage::MiniTypeMap;
use crate::{NodeId, NodeRef};
use core::panic;
use std::cell::SyncUnsafeCell;
use std::marker::PhantomPinned;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

/// For use by the #[node] macro, this drops runtime borrows.
pub struct BorrowDropper<'a>(&'a AtomicBool, PhantomPinned);

impl<'a> BorrowDropper<'a> {
    fn new(borrowed: &'a AtomicBool) -> Self {
        Self(borrowed, PhantomPinned)
    }
}

impl Drop for BorrowDropper<'_> {
    fn drop(&mut self) {
        self.0.store(false, Release);
    }
}

/// Contains a node's data and whether it is borrowed. [`T`] is a tuple of a
/// node's fields (#[ext] fields not included, those are stored as components).
pub struct NodeCell<T> {
    node: SyncUnsafeCell<T>,
    // Tracks whether this node is currently borrowed.
    borrowed: AtomicBool,
}

#[derive(Debug)]
pub struct NodeStorage(MiniTypeMap);

impl NodeStorage {
    pub fn new() -> Self {
        Self(MiniTypeMap::default())
    }

    /// Registers a node type if it does not exist already.
    pub fn register<T: NodeRef>(&mut self) {
        self.0.register::<T, _>();
    }

    /// Insert a node and corresponding components into storage.
    pub fn spawn<T>(&mut self, key: NodeKey, node: T::RecipeTuple) -> NodeId
    where
        T: NodeRef,
    {
        let node_type = self.mini_type_of::<T>();
        self.insert::<T, _>(
            key,
            NodeCell {
                node: SyncUnsafeCell::new(node),
                borrowed: AtomicBool::new(false),
            },
        );
        NodeId {
            node_type,
            instance: key,
        }
    }

    /// TODO
    #[allow(clippy::mut_from_ref)] // We do our own borrow checking.
    pub fn get_element<T>(&'_ self, id: NodeId) -> (&'_ mut T::RecipeTuple, BorrowDropper<'_>)
    where
        T: NodeRef,
    {
        let node_cell: &NodeCell<T::RecipeTuple> = unsafe {
            // TODO: ensure a custom NodeId can't be created to avoid a mismatch.
            self.get_unchecked::<T, _>(id.node_type, id.instance)
                .unwrap()
        };
        match node_cell
            .borrowed
            .compare_exchange(false, true, Acquire, Relaxed)
        {
            Ok(_) => (
                unsafe { node_cell.node.get().as_mut_unchecked() },
                BorrowDropper::new(&node_cell.borrowed),
            ),
            Err(_) => panic!("the same node should not be borrowed multiple times at once"),
        }
    }

    pub fn get_ids<T: NodeRef>(&self) -> impl ExactSizeIterator<Item = NodeId> {
        let node_type = self.mini_type_of::<T>();
        let keys = self.keys::<T, _>();
        keys.map(move |node_key: NodeKey| NodeId {
            node_type,
            instance: node_key,
        })
    }
}

impl Deref for NodeStorage {
    type Target = MiniTypeMap;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NodeStorage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
