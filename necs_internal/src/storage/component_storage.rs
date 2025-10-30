use super::key::NodeKey;
use crate::component::ComponentId;
use slotmap::SparseSecondaryMap;
use slotmap::sparse_secondary::ValuesMut;
use std::any::{Any, TypeId};
use std::cell::UnsafeCell;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ComponentStorage {
    // This is always a HashMap<TypeId, SparseSecondaryMap<ComponentId, T>>, but the
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
        self.map
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(UnsafeCell::new(SparseSecondaryMap::<NodeKey, T>::new())));
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

    /// Gets a mutable reference to an element of type `T` from the internal map using an unchecked operation.
    ///
    /// # Safety
    /// This function retrieves mutable references without enforcing borrowing rules, meaning the caller must guarantee there
    /// are no aliasing mutable or immutable references to the same data at the same time.
    ///
    /// # Type Parameters
    /// - `T`: The type of the component. Must satisfy the `'static`, `Send`, and `Sync` trait bounds.
    ///
    /// # Parameters
    /// - `id`: A reference to a `ComponentId<T>` that identifies the component whose mutable reference is being queried.
    ///   The key contained in `id` is used for the lookup within an internal `SparseSecondaryMap`.
    ///
    /// # Returns
    /// - `Option<&mut T>`: Returns an `Option` where:
    ///   - `Some(&mut T)` provides a mutable reference to the component if found.
    ///   - `None` is returned if the component with the given key does not exist in the map.
    ///
    /// # Panics
    /// This method will panic if the component type `T` has not been registered in the internal map prior to this call.
    /// The expectation is that all component types are registered beforehand.
    ///
    /// # Note
    /// - The `#[allow(clippy::mut_from_ref)]` attribute is used to suppress Clippy warnings related to creating mutable
    ///   references from an immutable reference, as this operation is explicitly intended in the context of this function.
    ///
    /// Use this method only when performance is critical and invariants can be manually guaranteed.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_element_unchecked<T: 'static + Send + Sync>(
        &self,
        id: &ComponentId<T>,
    ) -> Option<&mut T> {
        unsafe {
            self.map
                .get(&TypeId::of::<T>())
                .expect("the component type should be registered first")
                .as_mut_unchecked()
                .downcast_mut_unchecked::<SparseSecondaryMap<NodeKey, T>>()
                .get_mut(id.key)
        }
    }

    pub fn get_elements<T: 'static + Send + Sync>(
        &'a mut self,
    ) -> ValuesMut<'a, NodeKey, T> {
        unsafe {
            self.map
                .get_mut(&TypeId::of::<T>())
                .expect("the component type should be registered first")
                .get_mut()
                .downcast_mut_unchecked::<SparseSecondaryMap<NodeKey, T>>()
                .values_mut()
        }
    }
}
