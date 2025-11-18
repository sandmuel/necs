#![feature(downcast_unchecked)]
#![feature(tuple_trait)]
#![feature(unsafe_cell_access)]
#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]

pub use crate::node::{Field, NodeBuilder, NodeId, NodeRef, NodeTrait};
use crate::node_map::TypeMap;
pub use necs_macros::node;
use slotmap::SparseSecondaryMap;
use storage::Storage;

mod component;
pub use crate::node::Node;
pub use component::ComponentId;
pub use storage::BorrowDropper;
use storage::NodeKey;

mod node;
mod node_map;
pub mod storage;

pub type SubStorage<T> = SparseSecondaryMap<NodeKey, T>;

/// Storage for all nodes, related metadata, and functions.
pub struct World {
    pub(crate) storage: Storage,
    // Maps type ids to types, allowing us to work on nodes without knowing their types.
    pub node_map: TypeMap,
}

impl World {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_node<T>(&mut self)
    where
        T: NodeRef,
    {
        T::__register_node(&mut self.storage);
        self.node_map
            .register::<T, dyn Node, _>(self.storage.nodes.node_type_of::<T>(), |x| Box::new(x));
    }
    /*
    pub fn register_trait<T: NodeRef, Trait: NodeTrait + ?Sized>(&mut self) {
        self.node_map
            .register::<T, Trait, _>(self.storage.nodes.node_type_of::<T>(), |x| Box::new(x));
    }
     */
    pub fn spawn_node<T: NodeBuilder>(&mut self, node: T) -> NodeId {
        node.__move_to_storage(&mut self.storage)
    }
    pub fn get_node<T: NodeRef>(&self, id: NodeId) -> T::Instance<'_> {
        // The safety of this entirely depends on everything else not having issues.
        unsafe { T::__build_from_storage(&self.storage, id) }
    }
    /*
    pub fn get_nodes<T: NodeRef>(&self) -> Vec<T::Instance<'_>> {
        let ids = self.get_node_ids::<T>();

        let mut nodes = Vec::with_capacity(ids.len());

        for id in ids {
            unsafe { nodes.push(T::__build_from_storage(&self.storage, id)) }
        }

        nodes
    }
     */

    /*
    pub fn get_node_ids<T: NodeRef>(&self) -> Vec<NodeId> {
        let node_type = self.storage.nodes.node_type_of::<T>();
        self.storage
            .nodes
            .get_element::<T>()
            .keys()
            .map(|id| NodeId {
                node_type,
                instance: id,
            })
            .collect()
    }
     */

    /// Gets a node of type [T].
    ///
    /// This is similar to [`get_node`](World::get_node), but with [T] being a
    /// subtrait of [NodeTrait] (such as [Node]) implemented by the given node
    /// rather than the concrete type of the node.
    ///
    /// # Panics
    /// The node associated with the given [`NodeId`] must be of type [T].
    // TODO: Change panic doc ^
    pub fn get_node_resilient<T: 'static + NodeTrait + ?Sized>(&self, id: NodeId) -> Box<T> {
        // The safety of this entirely depends on everything else not having issues.
        self.node_map.get_node::<T>(&self.storage, id)
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            storage: Storage::new(),
            node_map: TypeMap::new(),
        }
    }
}
