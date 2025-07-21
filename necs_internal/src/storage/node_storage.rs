use crate::NodeId;
use slotmap::{DefaultKey, HopSlotMap};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::Tuple;
use std::ops::{Index, IndexMut};

type SubStorage<T> = HopSlotMap<DefaultKey, T>;

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

    pub fn register<T: 'static + Send + Sync>(&mut self) {
        if !self.map.contains_key(&TypeId::of::<T>()) {
            self.map
                .insert(TypeId::of::<T>(), Box::new(SubStorage::<T>::new()));
        }
    }

    pub fn spawn<T: 'static + Tuple + Send + Sync>(&mut self, node: T) -> NodeId {
        unsafe {
            NodeId {
                node_type: TypeId::of::<T>(),
                instance: self
                    .map
                    .get_mut(&TypeId::of::<T>())
                    .expect("node type should be registered first")
                    .downcast_mut_unchecked::<SubStorage<T>>()
                    .insert(node),
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
