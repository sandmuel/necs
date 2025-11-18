use crate::storage::NodeKey;
use std::marker::PhantomData;

/// Wrapper around [`NodeKey`], but with [`T`] included to make downcasting
/// straightforward.
pub struct ComponentId<T> {
    __type: PhantomData<T>,
    pub(crate) key: NodeKey,
}

impl<T> ComponentId<T> {
    /// Constructs a new [`ComponentId`] based on a given [`T`] representing the
    /// component's type, and [`NodeKey`].
    ///
    /// This should only be created where [T] is a component type present on the
    /// node which the given [`NodeKey`] refers to. Failing to meet these
    /// expectations can result in a panic where no entry of [NodeKey] can be
    /// found for [T].
    ///
    /// # Examples
    ///
    /// ```
    /// # use necs::{NodeId, ComponentId};
    /// # use necs::storage::NodeKey;
    /// # use slotmap::Key;
    /// let node_id: NodeId;
    /// # node_id = NodeId {
    /// #     node_type: 0,
    /// #     instance: NodeKey::null(),
    /// # };
    /// let instance = ComponentId::<u32>::new(node_id.instance);
    /// ```
    pub fn new(key: NodeKey) -> Self {
        Self {
            __type: PhantomData,
            key,
        }
    }
}
