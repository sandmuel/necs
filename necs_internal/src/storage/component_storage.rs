use super::{MiniTypeId, MiniTypeMap};
use crate::NodeKey;
use crate::component::ComponentId;
use std::cell::SyncUnsafeCell;

#[derive(Debug)]
pub struct ComponentStorage(MiniTypeMap);

impl<'a> ComponentStorage {
    pub(crate) fn new() -> Self {
        Self(MiniTypeMap::default())
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
    pub fn register<T>(&mut self) -> MiniTypeId
    where
        T: Send + Sync + 'static,
    {
        self.0.register::<T, _>()
    }

    /// Inserts the given component into storage.
    ///
    /// # Panics
    ///
    /// [`T`] must be registered with [`Self::register`] before calling this
    /// function.
    pub fn insert<T>(&mut self, key: NodeKey, component: T) -> ComponentId<T>
    where
        T: 'static + Send + Sync,
    {
        self.0.insert::<T, _>(key, SyncUnsafeCell::new(component));
        unsafe { ComponentId::new(self.0.mini_type_of::<T>(), key) }
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
    #[inline(always)]
    pub unsafe fn get_element_unchecked<T: 'static + Send + Sync>(
        &self,
        id: &ComponentId<T>,
    ) -> &mut T {
        // Safety: the type of the downcast is guaranteed to be correct since it is
        // based on the same type as the key. The caller must guarantee that another
        // reference to this component does not exist.
        unsafe {
            self.0
                .get_unchecked::<T, _>(id.into(), id.into())
                .unwrap_or_else(|| panic!("component with id {:?} not found", id))
                .get()
                .as_mut_unchecked()
        }
    }

    pub fn get_element<T: 'static + Send + Sync>(&self, id: &NodeKey) -> &'a mut T {
        unsafe {
            self.0
                .get_unchecked::<T, _>(self.0.mini_type_of::<T>(), *id)
                .unwrap_or_else(|| panic!("component with id {:?} not found", id))
                .get()
                .as_mut_unchecked()
        }
    }

    pub fn get_component<T: 'static + Send + Sync>(
        &mut self,
        id: &ComponentId<T>,
    ) -> Option<&mut T> {
        // Safety: the type of the downcast is guaranteed to be correct since it is on
        // the key.
        unsafe {
            self.0
                .get_mut_unchecked::<T, _>(id.into(), id.into())
                .map(|cell| cell.get_mut())
        }
    }

    pub fn get_components<T: 'static + Send + Sync>(
        &'a mut self,
    ) -> impl ExactSizeIterator<Item = &'a mut T> {
        self.0.values_mut::<T, _>().map(|cell| cell.get_mut())
    }
}
