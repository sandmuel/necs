use crate::ItemKey;
use crate::storage::{MiniTypeId, MiniTypeMap};
use crate::{NodeId, NodeRef};
use core::panic;
use slotmap::SlotMap;
use std::cell::SyncUnsafeCell;
use std::marker::PhantomPinned;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

/// For use by the `#[node]` macro, this drops runtime borrows.
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
pub struct RecipeTupleCell<T> {
    recipe_tuple: SyncUnsafeCell<T>,
    // Tracks whether this node is currently borrowed.
    borrowed: AtomicBool,
}

#[derive(Debug)]
pub struct NodeStorage {
    // This map only serves to generate unique keys.
    key_factory: SlotMap<ItemKey, ()>,
    nodes: MiniTypeMap,
}

impl NodeStorage {
    pub fn new() -> Self {
        Self {
            key_factory: SlotMap::default(),
            nodes: MiniTypeMap::default(),
        }
    }

    fn mint_key(&mut self) -> ItemKey {
        self.key_factory.insert(())
    }

    pub fn mini_type_of<T: NodeRef>(&self) -> MiniTypeId {
        self.nodes.mini_type_of::<T>()
    }

    /// Registers a node type if it does not exist already.
    pub fn register<T: NodeRef>(&mut self) {
        self.nodes.register::<T, _>();
    }

    /// Inserts a [T::RecipeTuple] into the storage.
    pub fn spawn<T>(&mut self, node: T::RecipeTuple) -> NodeId
    where
        T: NodeRef,
    {
        let key = self.mint_key();
        let node_type = self.nodes.mini_type_of::<T>();
        self.nodes.insert::<T, _>(
            key,
            RecipeTupleCell {
                recipe_tuple: SyncUnsafeCell::new(node),
                borrowed: AtomicBool::new(false),
            },
        );
        NodeId {
            node_type,
            instance: key,
        }
    }

    pub fn free<T>(&mut self, id: &NodeId) where T: NodeRef {
        self.nodes.remove::<T, _>(&id.instance)
    }

    /// TODO
    #[allow(clippy::mut_from_ref)] // We do our own borrow checking.
    pub fn get_element<T>(&'_ self, id: NodeId) -> (&'_ mut T::RecipeTuple, BorrowDropper<'_>)
    where
        T: NodeRef,
    {
        let node_cell: &RecipeTupleCell<T::RecipeTuple> = unsafe {
            // TODO: ensure a custom NodeId can't be created to avoid a mismatch.
            self.nodes
                .get_unchecked::<T, _>(id.node_type, id.instance).unwrap_or_else(|| panic!("node does not exist"))
        };
        match node_cell
            .borrowed
            .compare_exchange(false, true, Acquire, Relaxed)
        {
            Ok(_) => (
                unsafe { node_cell.recipe_tuple.get().as_mut_unchecked() },
                BorrowDropper::new(&node_cell.borrowed),
            ),
            Err(_) => panic!("the same node should not be borrowed multiple times at once"),
        }
    }

    pub fn get_ids<T: NodeRef>(&self) -> impl ExactSizeIterator<Item = NodeId> {
        let node_type = self.nodes.mini_type_of::<T>();
        let keys = self.nodes.keys::<T, _>();
        keys.map(move |node_key: &ItemKey| NodeId {
            node_type,
            instance: *node_key,
        })
    }

    pub unsafe fn get_node_cells_unchecked<T: NodeRef>(
        &self,
    ) -> impl ExactSizeIterator<Item = (&mut T::RecipeTuple, BorrowDropper<'_>)> {
        let node_cells = self.nodes.values::<T, _>();
        node_cells.map(|node_cell: &RecipeTupleCell<T::RecipeTuple>| {
            match node_cell
                .borrowed
                .compare_exchange(false, true, Acquire, Relaxed)
            {
                Ok(_) => (
                    unsafe { node_cell.recipe_tuple.get().as_mut_unchecked() },
                    BorrowDropper::new(&node_cell.borrowed),
                ),
                Err(_) => panic!("the same node should not be borrowed multiple times at once"),
            }
        })
    }
}
