use crate::node::NodeType;
use crate::storage::key::NodeKey;
use crate::{NodeId, NodeRef};
use core::panic;
use slotmap::SparseSecondaryMap;
use std::any::{Any, TypeId};
use std::cell::SyncUnsafeCell;
use std::collections::HashMap;
use std::marker::PhantomPinned;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

/// Contains a node's data and whether it is borrowed. [`T`] is a tuple of a
/// node's fields (#[ext] fields not included, those are stored as components).
struct NodeCell<T> {
    node: SyncUnsafeCell<T>,
    // Tracks whether this node is currently borrowed.
    borrowed: AtomicBool,
}

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

type SubStorage<T> = SparseSecondaryMap<NodeKey, NodeCell<T>>;

#[derive(Debug)]
pub struct NodeStorage {
    // Maps [`TypeId`]s to smaller node type identifiers.
    map_ids: HashMap<TypeId, NodeType>,
    // Contains a map of nodes for each node type.
    node_maps: Vec<Box<dyn Any + Send + Sync>>,
}

impl NodeStorage {
    pub fn new() -> Self {
        Self {
            map_ids: HashMap::new(),
            node_maps: Vec::new(),
        }
    }

    /// Registers a node type if it does not exist already.
    pub fn register<T: NodeRef>(&mut self) {
        if !self.map_ids.contains_key(&TypeId::of::<T>()) {
            let this_node_type = self.map_ids.len() as NodeType;
            // TODO make sure this gets called in release mode as well.
            let _ = NodeType::try_from(self.map_ids.len())
                .unwrap_or_else(|_| panic!("cannot register more than {} nodes", NodeType::MAX));
            self.map_ids.insert(TypeId::of::<T>(), this_node_type);
            self.node_maps
                .push(Box::new(SubStorage::<T::RecipeTuple>::new()));
        }
    }

    pub fn node_type_of<T: NodeRef>(&self) -> NodeType {
        *self
            .map_ids
            .get(&TypeId::of::<T>())
            .expect("the node type should be registered first")
    }

    /// Insert a node and corresponding components into storage.
    pub fn spawn<T: NodeRef>(&mut self, key: NodeKey, node: T::RecipeTuple) -> NodeId {
        let node_type = self.node_type_of::<T>();
        unsafe {
            self.node_maps[node_type as usize]
                // We checked that the node type is registered when defining node_type. The key used
                // corresponds to this type, so we know this is the correct type.
                .downcast_mut_unchecked::<SubStorage<T::RecipeTuple>>()
                .insert(
                    key,
                    NodeCell {
                        node: SyncUnsafeCell::new(node),
                        borrowed: AtomicBool::new(false),
                    },
                );
        }
        NodeId {
            node_type,
            instance: key,
        }
    }

    /// TODO
    #[allow(clippy::mut_from_ref)] // We do our own borrow checking.
    pub fn get_element<T: NodeRef>(
        &'_ self,
        id: NodeId,
    ) -> (&'_ mut T::RecipeTuple, BorrowDropper<'_>) {
        let node_type = self.node_type_of::<T>();
        let node_type = node_type as usize;
        let node_cell = unsafe {
            self.node_maps
                .get(node_type)
                .expect("the node type should be registered first")
                .as_ref()
                .downcast_ref_unchecked::<SubStorage<T::RecipeTuple>>()
                .get(id.instance)
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
}
