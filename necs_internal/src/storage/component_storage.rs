use crate::component::{ComponentId, ComponentKey};
use slotmap::HopSlotMap;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::ops::{Index, IndexMut};

#[derive(Debug)]
pub struct ComponentStorage {
    // This is always a BTreeMap<TypeId, HopSlotMap<ComponentId, T>>, but the HopSlotMap is made
    // dyn to avoid the need to downcast each T.
    map: HashMap<TypeId, Box<dyn Any>>,
}

impl ComponentStorage {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn register<T: 'static>(&mut self) {
        if !self.map.contains_key(&TypeId::of::<T>()) {
            self.map.insert(
                TypeId::of::<T>(),
                Box::new(HopSlotMap::<ComponentKey, T>::new()),
            );
        }
    }

    pub fn spawn<T: 'static>(&mut self, component: T) -> ComponentId<T> {
        unsafe {
            ComponentId::new(
                self.map
                    .get_mut(&TypeId::of::<T>())
                    .expect("component type should be registered first")
                    .downcast_mut_unchecked::<HopSlotMap<ComponentKey, T>>()
                    .insert(component),
            )
        }
    }
}

impl<T: 'static> Index<&ComponentId<T>> for ComponentStorage {
    type Output = T;

    fn index(&self, index: &ComponentId<T>) -> &Self::Output {
        let sub_storage = unsafe {
            self.map[&TypeId::of::<T>()].downcast_ref_unchecked::<HopSlotMap<ComponentKey, T>>()
        };
        &sub_storage[index.key]
    }
}

impl<T: 'static> IndexMut<&ComponentId<T>> for ComponentStorage {
    fn index_mut(&mut self, index: &ComponentId<T>) -> &mut Self::Output {
        let sub_storage = unsafe {
            self.map
                .get_mut(&TypeId::of::<T>())
                .expect("component type should be registered first")
                .downcast_mut_unchecked::<HopSlotMap<ComponentKey, T>>()
        };
        &mut sub_storage[index.key]
    }
}
