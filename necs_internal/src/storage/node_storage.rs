use crate::node::NodeType;
use crate::storage::key::NodeKey;
use crate::{NodeId, NodeRef};
use core::panic;
use slotmap::SparseSecondaryMap;
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::HashMap;

type SubStorage<T> = SparseSecondaryMap<NodeKey, T>;

#[derive(Debug)]
pub struct NodeStorage {
    map_ids: HashMap<TypeId, NodeType>,
    // TODO: add checks to ensure no double mut borrows
    node_maps: Vec<Box<UnsafeCell<dyn Any + Send + Sync>>>,
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
            self.node_maps.push(Box::new(UnsafeCell::new(
                SubStorage::<T::RecipeTuple>::new(),
            )));
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
                .get_mut()
                .downcast_mut_unchecked::<SubStorage<T::RecipeTuple>>()
                .insert(key, node);
        }
        NodeId {
            node_type,
            instance: key,
        }
    }

    /// TODO
    pub fn get_element<T>() {}

    /// TODO
    pub fn get_elements<T: NodeRef>(&self) -> &mut SubStorage<T::RecipeTuple> {
        let node_type = self
            .map_ids
            .get(&TypeId::of::<T>())
            .expect("the node type should be registered first");
        unsafe {
            self.node_maps
                .get(*node_type as usize)
                .expect("the node type should be registered first")
                .as_mut_unchecked()
                .downcast_mut_unchecked::<SubStorage<T::RecipeTuple>>()
        }
    }
}
