use crate::NodeId;
use crate::storage::map_key::MapKey;
use slotmap::HopSlotMap;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::Tuple;
use std::ops::{Index, IndexMut};

type SubStorage<T> = HopSlotMap<MapKey, T>;

#[derive(Debug)]
pub struct NodeStorage {
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl NodeStorage {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Registers a node type if it does not exist already.
    pub fn register<T: 'static + Send + Sync>(&mut self) {
        if !self.map.contains_key(&TypeId::of::<T>()) {
            self.map
                .insert(TypeId::of::<T>(), Box::new(SubStorage::<T>::new()));
        }
    }

    /// Insert a node and corresponding components into storage.
    pub fn spawn<T: 'static + Tuple + Send + Sync, F: FnOnce(MapKey) -> T>(
        &mut self,
        f: F,
    ) -> NodeId {
        unsafe {
            NodeId {
                node_type: TypeId::of::<T>(),
                instance: self
                    .map
                    .get_mut(&TypeId::of::<T>())
                    .expect("the node type should be registered first")
                    .downcast_mut_unchecked::<SubStorage<T>>()
                    .insert_with_key(|key| f(key)),
            }
        }
    }
}

impl Index<TypeId> for NodeStorage {
    type Output = dyn Any;

    fn index(&self, index: TypeId) -> &Self::Output {
        &*self.map[&index]
    }
}

impl IndexMut<TypeId> for NodeStorage {
    fn index_mut(&mut self, index: TypeId) -> &mut Self::Output {
        &mut **self
            .map
            .get_mut(&index)
            .expect("node should not be retrieved after being freed")
    }
}
