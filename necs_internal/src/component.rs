use crate::storage::map_key::MapKey;
use std::marker::PhantomData;

/// Wrapper around [`ComponentKey`], but with [`T`] included to make downcasting
/// straightforward.
pub struct ComponentId<T> {
    __type: PhantomData<T>,
    pub(crate) key: MapKey,
}

impl<T> ComponentId<T> {
    pub fn new(key: MapKey) -> Self {
        Self {
            __type: PhantomData::default(),
            key,
        }
    }
}
