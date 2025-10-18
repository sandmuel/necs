#![feature(downcast_unchecked)]
#![feature(tuple_trait)]

pub use crate::node::{Field, NodeBuilder, NodeId, NodeRef, NodeTrait};
use crate::node_map::TypeMap;
use crate::storage::Storage;
use slotmap::SparseSecondaryMap;

mod component;
pub use crate::node::Node;
use crate::storage::key::NodeKey;
pub use component::ComponentId;

mod node;
mod node_map;
pub mod storage;

pub type SubStorage<T> = SparseSecondaryMap<NodeKey, T>;

/// Storage for all nodes, related metadata, and functions.
pub struct World {
    // Maps type ids to types, allowing us to work on nodes without knowing their types.
    pub node_map: TypeMap,
    storage: Storage,
    // TODO: Keep track of borrowed components and nodes.
}

impl World {
    pub fn new() -> Self {
        Self {
            node_map: TypeMap::new(),
            storage: Storage::new(),
        }
    }
    pub fn register_node<T>(&mut self)
    where
        T: NodeRef,
    {
        T::__register_node(&mut self.storage);
        self.node_map
            .register::<T, dyn Node, _>(self.storage.nodes.node_type_of::<T>(), |x| Box::new(x));
    }
    pub fn spawn_node<T: NodeBuilder>(&mut self, node: T) -> NodeId {
        let id = node.__move_to_storage(&mut self.storage);
        id
    }
    pub fn get_node<T: NodeRef>(&mut self, id: NodeId) -> T::Instance<'_> {
        // The safety of this entirely depends on everything else not having issues.
        unsafe { T::__build_from_storage(&mut self.storage, id) }
    }
    /*
    pub fn get_nodes<T: NodeRef>(&mut self) -> Vec<T::Instance<'_>> {
        let ids = self.get_node_ids::<T>();

        let mut nodes = Vec::with_capacity(ids.len());

        for id in ids {
            unsafe { nodes.push(T::__build_from_storage(&mut self.storage, id)) }
        }

        nodes
    }
     */

    pub fn get_node_ids<T: NodeRef>(&mut self) -> Vec<NodeId> {
        let node_type = self.storage.nodes.node_type_of::<T>();
        self.storage.nodes[node_type]
            .downcast_mut::<SubStorage<T::RecipeTuple>>()
            // TODO give this a proper error message.
            .unwrap()
            .keys()
            .map(|id| NodeId {
                node_type,
                instance: id,
            })
            .collect()
    }

    /// Gets a node of type [T].
    ///
    /// This is similar to [`get_node`](World::get_node), but with [T] being a
    /// subtrait of [NodeTrait] (such as [Node]) implemented by the given node
    /// rather than the concrete type of the node.
    ///
    /// # Panics
    /// The node associated with the given [`NodeId`] must be of type [T].
    // TODO: Change panic doc ^
    pub fn get_node_resilient<T: 'static + NodeTrait + ?Sized>(&mut self, id: NodeId) -> Box<T> {
        // The safety of this entirely depends on everything else not having issues.
        self.node_map.get_node::<T>(&mut self.storage, id)
    }
}
