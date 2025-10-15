use crate::component::ComponentId;
use crate::storage::map_key::MapKey;
use slotmap::SparseSecondaryMap;
use slotmap::sparse_secondary::ValuesMut;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct ComponentStorage {
    // This is always a BTreeMap<TypeId, SparseSecondaryMap<ComponentId, T>>, but the
    // SparseSecondaryMap is made dyn to avoid the need to downcast each value.
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
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
                Box::new(SparseSecondaryMap::<MapKey, T>::new()),
            );
        }
    }

    pub fn spawn<T: 'static + Send + Sync>(&mut self, key: MapKey, component: T) -> ComponentId<T> {
        unsafe {
            self.map
                .get_mut(&TypeId::of::<T>())
                .expect("the component type should be registered first")
                .downcast_mut_unchecked::<SparseSecondaryMap<MapKey, T>>()
                .insert(key, component);
            ComponentId::new(key)
        }
    }

    pub fn get_all<T: 'static + Send + Sync>(&'a mut self) -> ValuesMut<'a, MapKey, T> {
        unsafe {
            self.map
                .get_mut(&TypeId::of::<T>())
                .expect("the component type should be registered first")
                .downcast_mut_unchecked::<SparseSecondaryMap<MapKey, T>>()
                .values_mut()
                .into_iter()
        }
    }
}

impl<T: 'static + Send + Sync> Index<&ComponentId<T>> for ComponentStorage {
    type Output = T;

    fn index(&self, index: &ComponentId<T>) -> &Self::Output {
        let sub_storage = unsafe {
            self.map[&TypeId::of::<T>()].downcast_ref_unchecked::<SparseSecondaryMap<MapKey, T>>()
        };
        &sub_storage[index.key]
    }
}

impl<T: 'static + Send + Sync> IndexMut<&ComponentId<T>> for ComponentStorage {
    fn index_mut(&mut self, index: &ComponentId<T>) -> &mut Self::Output {
        let sub_storage = unsafe {
            self.map
                .get_mut(&TypeId::of::<T>())
                .expect("the component type should be registered first")
                .downcast_mut_unchecked::<SparseSecondaryMap<MapKey, T>>()
        };
        &mut sub_storage[index.key]
    }
}
