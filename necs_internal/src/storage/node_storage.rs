use crate::node::NodeType;
use crate::storage::key::NodeKey;
use crate::{NodeId, NodeRef};
use core::panic;
use slotmap::SparseSecondaryMap;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::Tuple;
use std::ops::{Index, IndexMut};

type SubStorage<T> = SparseSecondaryMap<NodeKey, T>;

#[derive(Debug)]
pub struct NodeStorage {
    map_ids: HashMap<TypeId, NodeType>,
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
    pub fn register<T: NodeRef + Send + Sync>(&mut self) {
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
    pub fn spawn<T: NodeRef + Send + Sync>(
        &mut self,
        key: NodeKey,
        node: T::RecipeTuple,
    ) -> NodeId {
        let node_type = self.node_type_of::<T>();
        unsafe {
            self.node_maps[node_type as usize]
                // We checked that the node type is registered when defining node_type.
                // The key used corresponds to this type, so we know this is the correct type.
                .downcast_mut_unchecked::<SubStorage<T::RecipeTuple>>()
                .insert(key, node);
        }
        NodeId {
            node_type,
            instance: key,
        }
    }
}

impl Index<NodeType> for NodeStorage {
    type Output = dyn Any;

    fn index(&self, index: NodeType) -> &Self::Output {
        &*self
            .node_maps
            .get(index as usize)
            .expect("the node type should be registered first")
    }
}

impl IndexMut<NodeType> for NodeStorage {
    fn index_mut(&mut self, index: NodeType) -> &mut Self::Output {
        &mut **self
            .node_maps
            .get_mut(index as usize)
            .expect("the node type should be registered first")
    }
}
