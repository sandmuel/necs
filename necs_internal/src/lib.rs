#![feature(downcast_unchecked)]
#![feature(tuple_trait)]

pub use crate::node::{Field, NodeTrait, NodeBuilder, NodeId, NodeRef};
use crate::storage::Storage;
use crate::type_map::TypeMap;
use slotmap::{DefaultKey, HopSlotMap};
use std::any::TypeId;

mod component;
pub use component::ComponentId;
use crate::node::Node;

mod node;
pub mod storage;
mod type_map;

pub type SubStorage<T> = HopSlotMap<DefaultKey, T>;

pub struct World {
    // Maps type ids to types, allowing us to work on Nodes without knowing their types.
    pub type_map: TypeMap,
    pub storage: Storage,
    // TODO: Keep track of borrowed components and nodes.
}

impl World {
    pub fn new() -> Self {
        Self {
            type_map: TypeMap::new(),
            storage: Storage::new(),
        }
    }
    pub fn register_node<T: 'static + NodeRef>(&mut self) {
        println!("Added {:?} to typemap", TypeId::of::<T::RecipeTuple>());
        self.type_map.register::<T>();
        T::__register_node(&mut self.storage);
    }
    pub fn spawn_node<T: NodeBuilder>(&mut self, node: T) -> NodeId {
        unsafe {
            let id = node.__move_to_storage(&mut self.storage);
            id
        }
    }
    pub fn get_node<T: 'static + NodeRef>(&mut self, id: NodeId) -> T {
        // The safety of this entirely depends on everything else not having issues.
        unsafe { T::__build_from_storage(&mut self.storage, id) }
    }
    /// Gets a node of type T.
    ///
    /// This is similar to `get_node`, but it doesn't require T to implement
    /// NodeRef.
    ///
    /// # Safety
    /// The node associated with the given [NodeId] must be of type T.
    pub fn get_node_resilient<T: 'static + NodeTrait + ?Sized>(&mut self, id: NodeId) -> Box<T> {
        // The safety of this entirely depends on everything else not having issues.
        // TODO fix this. node_type is currently the RecipeTuple rather than the actual
        // node type.
        unsafe { self.type_map.get_node::<Box<T>>(&mut self.storage, id) }
    }
}
