#![feature(downcast_unchecked)]
#![feature(unsafe_cell_access)]
#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]

use rustc_hash::FxHashMap as HashMap;
pub use crate::node::{Field, NodeBuilder, NodeId, NodeRef, NodeTrait};
use crate::trait_map::TraitMap;
pub use necs_macros::node;
use slotmap::SparseSecondaryMap;
use storage::Storage;

mod component;
pub use crate::node::Node;
pub use component::ComponentId;
pub use storage::BorrowDropper;
pub use storage::ItemKey;
pub use relations::Relations;

mod node;
pub mod storage;
mod trait_map;
mod relations;

pub type SubStorage<T> = SparseSecondaryMap<ItemKey, T>;

/// Storage for all nodes, related metadata, and functions.
#[derive(Debug)]
pub struct World {
    pub(crate) storage: Storage,
    // Maps TypeIds to types, allowing us to work on nodes without knowing their types.
    trait_map: TraitMap,
    // TODO: I should really give this a better name.
    pub community: HashMap<ItemKey, Relations>,
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
        self.trait_map
            .register::<T, dyn Node, _>(self.storage.nodes.mini_type_of::<T>(), |x| Box::new(x));
    }
    pub fn register_trait<T, Trait, F>(&mut self, to_trait_obj: F)
    where
        T: NodeRef + Node,
        Trait: NodeTrait + ?Sized + 'static,
        F: Fn(T::Instance<'static>) -> Box<Trait> + Send + Sync + 'static,
    {
        self.trait_map
            .register::<T, Trait, _>(self.storage.nodes.mini_type_of::<T>(), to_trait_obj);
    }
    pub fn spawn_node<T: NodeBuilder>(&mut self, node: T) -> NodeId {
        let node_id = node.__move_to_storage(&mut self.storage);
        self.community.insert(node_id.instance, Relations::new(None));
        node_id
    }
    pub fn free_node<T>(&mut self, node_id: &NodeId) where T: NodeRef {
        self.storage.nodes.free::<T>(node_id);
    }
    pub fn get_node<T: NodeRef>(&self, id: NodeId) -> T::Instance<'_> {
        // The safety of this entirely depends on everything else not having issues.
        let (recipe_tuple, borrow_dropper) = self.storage.nodes.get_element::<T>(id);
        unsafe { T::__build_from_storage(recipe_tuple, borrow_dropper, &self.storage, id) }
    }
    pub fn get_nodes<T: NodeRef>(&self) -> Vec<T::Instance<'_>> {
        let ids = self.get_node_ids::<T>();

        let mut nodes = Vec::with_capacity(ids.len());

        let recipe_tuples = unsafe { self.storage.nodes.get_node_cells_unchecked::<T>() };

        for ((recipe_tuple, borrow), id) in recipe_tuples.zip(ids) {
            unsafe {
                nodes.push(T::__build_from_storage(
                    recipe_tuple,
                    borrow,
                    &self.storage,
                    id,
                ))
            }
        }

        nodes
    }
    pub fn get_node_ids<T: NodeRef>(&self) -> impl ExactSizeIterator<Item = NodeId> {
        self.storage.nodes.get_ids::<T>()
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
    pub fn get_node_resilient<T: 'static + NodeTrait + ?Sized>(&self, id: NodeId) -> Box<T> {
        self.trait_map.get_node::<T>(&self.storage, id)
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            storage: Storage::new(),
            trait_map: TraitMap::new(),
            community: HashMap::default(),
        }
    }
}
