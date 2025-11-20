use super::key::NodeKey;
use crate::component::ComponentId;
use rustc_hash::FxHashMap as HashMap;
use slotmap::SparseSecondaryMap;
use slotmap::sparse_secondary::ValuesMut;
use std::any::{Any, TypeId};
use std::cell::SyncUnsafeCell;

#[derive(Debug)]
pub struct ComponentStorage {
    // This is always a HashMap<TypeId, SparseSecondaryMap<ComponentId, T>>, but the
    // SparseSecondaryMap is made dyn to avoid the need to downcast each value.
    map: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl<'a> ComponentStorage {
    pub(crate) fn new() -> Self {
        Self {
            map: HashMap::default(),
        }
    }

    /// Registers [T] as a component type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use necs::World;
    /// # use necs::storage::Storage;
    /// #
    /// # let mut world = Storage::new();
    ///
    /// let mut components = world.components;
    ///
    /// components.register::<u32>();
    /// ```
    pub fn register<T: 'static + Send + Sync>(&mut self) {
        self.map
            .entry(TypeId::of::<T>())
            .or_insert(Box::new(
                SparseSecondaryMap::<NodeKey, SyncUnsafeCell<T>>::new(),
            ));
    }

    /// Inserts the given component into storage.
    ///
    /// # Panics
    ///
    /// [`T`] must be registered with [`Self::register`] before calling this
    /// function.
    pub fn insert<T: 'static + Send + Sync>(
        &mut self,
        key: NodeKey,
        component: T,
    ) -> ComponentId<T> {
        // Safety: the type of the downcast is guaranteed to be correct since it is
        // based on the same type as the key.
        unsafe {
            self.map
                .get_mut(&TypeId::of::<T>())
                .expect("the component type should be registered first")
                .downcast_mut_unchecked::<SparseSecondaryMap<NodeKey, T>>()
                .insert(key, component);
            ComponentId::new(key)
        }
    }

    /// Gets a mutable reference to an element of type `T` from the internal map
    /// using an unchecked operation.
    ///
    /// # Safety
    ///
    /// This function retrieves mutable references without enforcing borrowing
    /// rules, meaning the caller must guarantee there are no aliasing
    /// mutable or immutable references to the same data at the same time.
    ///
    /// # Returns
    /// - `Option<&mut T>`: Returns an `Option` where:
    ///   - `Some(&mut T)` provides a mutable reference to the component if
    ///     found.
    ///   - `None` is returned if the component with the given key does not
    ///     exist in the map.
    ///
    /// # Panics
    ///
    /// This method will panic if the component type `T` has not been registered
    /// in the internal map prior to this call.
    ///
    /// # Note
    /// - The `#[allow(clippy::mut_from_ref)]` attribute is used to suppress
    ///   Clippy warnings related to creating mutable references from an
    ///   immutable reference, as this operation is explicitly intended in the
    ///   context of this function.
    ///
    /// Use this method only when performance is critical and invariants can be
    /// manually guaranteed.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_element_unchecked<T: 'static + Send + Sync>(
        &self,
        id: &ComponentId<T>,
    ) -> &mut T {
        // Safety: the type of the downcast is guaranteed to be correct since it is
        // based on the same type as the key. The caller must guarantee that another
        // reference to this component does not exist.
        unsafe {
            self.map
                .get(&TypeId::of::<T>())
                .expect("the component type should be registered first")
                .downcast_ref_unchecked::<SparseSecondaryMap<NodeKey, SyncUnsafeCell<T>>>()
                .get_unchecked(id.into())
                .get()
                .as_mut_unchecked()
        }
    }

    pub fn get_element<T: 'static + Send + Sync>(&mut self, id: &ComponentId<T>) -> Option<&mut T> {
        // Safety: the type of the downcast is guaranteed to be correct since it is
        // based on the same type as the key.
        unsafe {
            self.map
                .get_mut(&TypeId::of::<T>())
                .expect("the component type should be registered first")
                .downcast_mut_unchecked::<SparseSecondaryMap<NodeKey, T>>()
                .get_mut(id.into())
        }
    }

    pub fn get_elements<T: 'static + Send + Sync>(&'a mut self) -> ValuesMut<'a, NodeKey, T> {
        // Safety: the type of the downcast is guaranteed to be correct since it is
        // based on the same type as the key.
        unsafe {
            self.map
                .get_mut(&TypeId::of::<T>())
                .expect("the component type should be registered first")
                .downcast_mut_unchecked::<SparseSecondaryMap<NodeKey, T>>()
                .values_mut()
        }
    }
}
