use super::key::NodeKey;
use crate::component::ComponentId;
use slotmap::SparseSecondaryMap;
use slotmap::sparse_secondary::ValuesMut;
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ComponentStorage {
    // This is always a BTreeMap<TypeId, SparseSecondaryMap<ComponentId, T>>, but the
    // SparseSecondaryMap is made dyn to avoid the need to downcast each value.
    map: HashMap<TypeId, Box<UnsafeCell<dyn Any + Send + Sync>>>,
}

impl<'a> ComponentStorage {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register<T: 'static + Send + Sync>(&mut self) {
        if !self.map.contains_key(&TypeId::of::<T>()) {
            self.map.insert(
                TypeId::of::<T>(),
                Box::new(UnsafeCell::new(SparseSecondaryMap::<NodeKey, T>::new())),
            );
        }
    }

    pub fn insert<T: 'static + Send + Sync>(
        &mut self,
        key: NodeKey,
        component: T,
    ) -> ComponentId<T> {
        unsafe {
            self.map
                .get_mut(&TypeId::of::<T>())
                .expect("the component type should be registered first")
                .get_mut()
                .downcast_mut_unchecked::<SparseSecondaryMap<NodeKey, T>>()
                .insert(key, component);
            ComponentId::new(key)
        }
    }

    pub unsafe fn get_element<T: 'static + Send + Sync>(&self, id: ComponentId<T>) -> &T {
        unsafe {
            self.map
                .get(&TypeId::of::<T>())
                .expect("the component type should be registered first")
                .as_mut_unchecked()
                .downcast_mut_unchecked::<SparseSecondaryMap<NodeKey, T>>()
                .get_mut(id.key)
                .unwrap() // TODO: return an error instead of panicking.
        }
    }

    pub unsafe fn get_elements<T: 'static + Send + Sync>(
        &'a mut self,
    ) -> ValuesMut<'a, NodeKey, T> {
        unsafe {
            self.map
                .get_mut(&TypeId::of::<T>())
                .expect("the component type should be registered first")
                .get_mut()
                .downcast_mut_unchecked::<SparseSecondaryMap<NodeKey, T>>()
                .values_mut()
                .into_iter()
        }
    }
}
