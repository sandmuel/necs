use crate::storage::node_storage::RecipeTupleCell;
use crate::{MiniTypeId, NodeKey, NodeRef};
use rustc_hash::FxHashMap as HashMap;
use slotmap::SparseSecondaryMap;
use std::any::{Any, TypeId, type_name};
use std::cell::SyncUnsafeCell;

#[derive(Debug, Default)]
pub struct MiniTypeMap {
    id_map: HashMap<TypeId, MiniTypeId>,
    data: Vec<Box<dyn Any + Send + Sync>>,
}

impl MiniTypeMap {
    pub fn register<T: MiniTypeMapKey<D>, D>(&mut self)
    where
        T::Value: Send + Sync,
    {
        let type_id = TypeId::of::<T>();
        if !self.id_map.contains_key(&type_id) {
            let mini_type_id = MiniTypeId::from(self.id_map.len());
            self.id_map.insert(type_id, mini_type_id);
            let sub_map: SparseSecondaryMap<NodeKey, T::Value> = SparseSecondaryMap::default();
            self.data.push(Box::new(sub_map));
        }
    }

    pub fn mini_type_of<T: 'static>(&self) -> MiniTypeId {
        *self.id_map.get(&TypeId::of::<T>()).unwrap_or_else(|| {
            panic!(
                "cannot get MiniTypeId for unregistered type {:?}",
                type_name::<T>()
            )
        })
    }

    pub fn insert<T: MiniTypeMapKey<D>, D>(&mut self, key: NodeKey, item: T::Value) {
        let mini_type_id = self.mini_type_of::<T>();
        let sub_map = unsafe {
            self.data[mini_type_id.index()]
                .as_mut()
                // SAFETY: We know this is the correct type because both the key and value are
                // derived from the same type.
                .downcast_unchecked_mut::<SparseSecondaryMap<NodeKey, T::Value>>()
        };
        sub_map.insert(key, item);
    }

    pub fn keys<T: MiniTypeMapKey<D>, D>(&self) -> impl ExactSizeIterator<Item = NodeKey> {
        let mini_type_id = self.mini_type_of::<T>();
        let sub_map = unsafe {
            self.data[mini_type_id.index()]
                .as_ref()
                // SAFETY: We know this is the correct type because both the key and value are
                // derived from the same type.
                .downcast_unchecked_ref::<SparseSecondaryMap<NodeKey, T::Value>>()
        };
        sub_map.keys()
    }

    pub fn values<T: MiniTypeMapKey<D>, D>(&self) -> impl ExactSizeIterator<Item = &T::Value> {
        let mini_type_id = self.mini_type_of::<T>();
        let sub_map = unsafe {
            self.data[mini_type_id.index()]
                .as_ref()
                // SAFETY: We know this is the correct type because both the key and value are
                // derived from the same type.
                .downcast_unchecked_ref::<SparseSecondaryMap<NodeKey, T::Value>>()
        };
        sub_map.values()
    }

    pub fn values_mut<T: MiniTypeMapKey<D>, D>(
        &mut self,
    ) -> impl ExactSizeIterator<Item = &mut T::Value> {
        let mini_type_id = self.mini_type_of::<T>();
        let sub_map = unsafe {
            self.data[mini_type_id.index()]
                .as_mut()
                // SAFETY: We know this is the correct type because both the key and value are
                // derived from the same type.
                .downcast_unchecked_mut::<SparseSecondaryMap<NodeKey, T::Value>>()
        };
        sub_map.values_mut()
    }

    /// SAFETY: The caller must ensure that the MiniTypeId corresponds to [T].
    pub unsafe fn get_unchecked<T: MiniTypeMapKey<D>, D>(
        &self,
        mini_type_id: MiniTypeId,
        key: NodeKey,
    ) -> Option<&T::Value> {
        let sub_map = unsafe {
            self.data[mini_type_id.index()]
                .as_ref()
                // As long as this function's invariant is upheld, this is safe.
                .downcast_unchecked_ref::<SparseSecondaryMap<NodeKey, T::Value>>()
        };
        sub_map.get(key)
    }

    pub unsafe fn get_mut_unchecked<T: MiniTypeMapKey<D>, D>(
        &mut self,
        mini_type_id: MiniTypeId,
        key: NodeKey,
    ) -> Option<&mut T::Value> {
        let sub_map = unsafe {
            self.data[mini_type_id.index()]
                .as_mut()
                // As long as this function's invariant is upheld, this is safe.
                .downcast_unchecked_mut::<SparseSecondaryMap<NodeKey, T::Value>>()
        };
        sub_map.get_mut(key)
    }
}

pub trait MiniTypeMapKey<Disambiguator>: 'static {
    type Value: Send + Sync + 'static;
}
pub(crate) struct OwnValue;
impl<T: Send + Sync + 'static> MiniTypeMapKey<OwnValue> for T {
    type Value = SyncUnsafeCell<Self>;
}
pub(crate) struct RecipeTuple;
impl<T: NodeRef> MiniTypeMapKey<RecipeTuple> for T {
    type Value = RecipeTupleCell<T::RecipeTuple>;
}
