use slotmap::DefaultKey;
use std::marker::PhantomData;

/// Key to a specific component.
pub(crate) type ComponentKey = DefaultKey;

/// Wrapper around [`ComponentKey`], but with [`T`] included to make downcasting
/// straightforward.
pub struct ComponentId<T> {
    __type: PhantomData<T>,
    pub(crate) key: ComponentKey,
}

impl<T> ComponentId<T> {
    pub fn new(key: ComponentKey) -> Self {
        Self {
            __type: PhantomData::default(),
            key,
        }
    }
}
