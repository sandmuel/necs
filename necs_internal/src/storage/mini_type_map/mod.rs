use crate::NodeRef;
use crate::storage::node_storage::RecipeTupleCell;
use rustc_hash::FxHashMap as HashMap;
use std::any::{Any, TypeId, type_name};
use std::cell::SyncUnsafeCell;
use std::fmt::Debug;

mod mini_type_id;
pub use mini_type_id::MiniTypeId;

mod key;
pub use key::ItemKey;

#[cold]
#[inline(never)]
fn type_not_registered<T>() -> ! {
    panic!(
        "cannot get MiniTypeId for unregistered type {:?}",
        type_name::<T>()
    )
}

#[derive(Debug, Default)]
pub struct MiniTypeMap {
    id_map: HashMap<TypeId, MiniTypeId>,
    data: Vec<Box<dyn Any + Send + Sync>>,
}

impl MiniTypeMap {
    /// Registers type [`T`] to this map.
    ///
    /// We can get the [`MiniTypeId`] of [`T`] using [`Self::mini_type_of`].
    pub fn register<T: MiniTypeMapKey<D>, D>(&mut self) -> MiniTypeId {
        let type_id = TypeId::of::<T>();
        let next_idx = self.id_map.len();
        let entry = self.id_map.entry(type_id).or_insert_with(|| {
            let mini_type_id = MiniTypeId::from(next_idx);
            let sub_map: HashMap<ItemKey, T::Value> = HashMap::default();
            self.data.push(Box::new(sub_map));
            mini_type_id
        });
        *entry
    }

    /// Returns the [`MiniTypeId`] corresponding to [`T`].
    #[inline]
    pub fn mini_type_of<T: 'static>(&self) -> MiniTypeId {
        *self
            .id_map
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| type_not_registered::<T>())
    }

    #[inline]
    pub fn insert<T: MiniTypeMapKey<D>, D>(&mut self, key: ItemKey, item: T::Value) {
        let mini_type_id = self.mini_type_of::<T>();
        let sub_map = unsafe {
            // SAFETY: The call to mini_type_of() would have panicked if the type wasn't
            // registered.
            self.data
                .get_unchecked_mut(mini_type_id.index())
                // SAFETY: We know this is the correct type because both the key and value are
                // derived from the same type.
                .downcast_unchecked_mut::<HashMap<ItemKey, T::Value>>()
        };
        sub_map.insert(key, item);
    }

    #[inline]
    pub fn keys<T: MiniTypeMapKey<D>, D>(&self) -> impl ExactSizeIterator<Item = &ItemKey> {
        let mini_type_id = self.mini_type_of::<T>();
        let sub_map = unsafe {
            // SAFETY: The call to mini_type_of() would have panicked if the type wasn't
            // registered.
            self.data
                .get_unchecked(mini_type_id.index())
                // SAFETY: We know this is the correct type because both the key and value are
                // derived from the same type.
                .downcast_unchecked_ref::<HashMap<ItemKey, T::Value>>()
        };
        sub_map.keys()
    }

    #[inline]
    pub fn values<T: MiniTypeMapKey<D>, D>(&self) -> impl ExactSizeIterator<Item = &T::Value> {
        let mini_type_id = self.mini_type_of::<T>();
        let sub_map = unsafe {
            // SAFETY: The call to mini_type_of() would have panicked if the type wasn't
            // registered.
            self.data
                .get_unchecked(mini_type_id.index())
                // SAFETY: We know this is the correct type because both the key and value are
                // derived from the same type.
                .downcast_unchecked_ref::<HashMap<ItemKey, T::Value>>()
        };
        sub_map.values()
    }

    #[inline]
    pub fn values_mut<T: MiniTypeMapKey<D>, D>(
        &mut self,
    ) -> impl ExactSizeIterator<Item = &mut T::Value> {
        let mini_type_id = self.mini_type_of::<T>();
        let sub_map = unsafe {
            // SAFETY: The call to mini_type_of() would have panicked if the type wasn't
            // registered.
            self.data
                .get_unchecked_mut(mini_type_id.index())
                // SAFETY: We know this is the correct type because both the key and value are
                // derived from the same type.
                .downcast_unchecked_mut::<HashMap<ItemKey, T::Value>>()
        };
        sub_map.values_mut()
    }

    #[inline]
    pub unsafe fn get_unchecked<T: MiniTypeMapKey<D>, D>(
        &self,
        mini_type_id: MiniTypeId,
        key: ItemKey,
    ) -> Option<&T::Value> {
        let sub_map = unsafe {
            self.data
                .get(mini_type_id.index())
                .unwrap_or_else(|| type_not_registered::<T>())
                // SAFETY: the caller guarantees T corresponds to mini_type_id.
                .downcast_unchecked_ref::<HashMap<ItemKey, T::Value>>()
        };
        sub_map.get(&key)
    }

    #[inline]
    pub unsafe fn get_mut_unchecked<T: MiniTypeMapKey<D>, D>(
        &mut self,
        mini_type_id: MiniTypeId,
        key: ItemKey,
    ) -> Option<&mut T::Value> {
        let sub_map = unsafe {
            self.data
                .get_mut(mini_type_id.index())
                .unwrap_or_else(|| type_not_registered::<T>())
                // SAFETY: the caller guarantees T corresponds to mini_type_id.
                .downcast_unchecked_mut::<HashMap<ItemKey, T::Value>>()
        };
        sub_map.get_mut(&key)
    }
}

pub trait MiniTypeMapKey<Disambiguator>: 'static {
    type Value: Send + Sync + 'static;
}
pub struct OwnValue;
impl<T: Send + Sync + 'static> MiniTypeMapKey<OwnValue> for T {
    type Value = SyncUnsafeCell<Self>;
}
pub struct RecipeTuple;
impl<T: NodeRef> MiniTypeMapKey<RecipeTuple> for T {
    type Value = RecipeTupleCell<T::RecipeTuple>;
}
