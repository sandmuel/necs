use crate::storage::NodeKey;
use std::marker::PhantomData;

/// A wrapper around [`NodeKey`], but with [`T`] included to support
/// downcasting.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ComponentId<T> {
    key: NodeKey,
    _marker: PhantomData<T>,
}

impl<T> ComponentId<T> {
    /// Constructs a new [`ComponentId`] based on a given [`T`] representing the
    /// component's type, and [`NodeKey`].
    ///
    /// This should only be created where [`T`] is a component type present on
    /// the node which the given [`NodeKey`] refers to. Failing to meet these
    /// expectations can result in a panic where no entry of [`NodeKey`] can be
    /// found for [`T`].
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
            key,
            _marker: PhantomData,
        }
    }
}

impl<T> From<&ComponentId<T>> for NodeKey {
    fn from(component_id: &ComponentId<T>) -> Self {
        component_id.key
    }
}
