use crate::storage::key::NodeKey;
use std::marker::PhantomData;

/// Wrapper around [`ComponentKey`], but with [`T`] included to make downcasting
/// straightforward.
pub struct ComponentId<T> {
    __type: PhantomData<T>,
    pub(crate) key: NodeKey,
}

impl<T> ComponentId<T> {
    pub fn new(key: NodeKey) -> Self {
        Self {
            __type: PhantomData::default(),
            key,
        }
    }
}
