use crate::storage::{MiniTypeId, NodeKey};
use std::fmt::{Debug, Display};
use std::marker::PhantomData;

/// A wrapper around [`NodeKey`], along with [`T`] and [`MiniTypeId`] for
/// efficient downcasting.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct ComponentId<T> {
    component_type: MiniTypeId,
    key: NodeKey,
    _marker: PhantomData<T>,
}

impl<T> Debug for ComponentId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.key)
    }
}

impl<T> Display for ComponentId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.key)
    }
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
    /// # use necs::NodeKey;
    /// # use slotmap::Key;
    /// use necs_internal::storage::MiniTypeId;
    /// let node_id: NodeId;
    /// # node_id = NodeId {
    /// #     node_type: MiniTypeId::from(0),
    /// #     instance: NodeKey::null(),
    /// # };
    /// let instance = unsafe { ComponentId::<u32>::new(node_id.node_type, node_id.instance) };
    /// ```
    pub unsafe fn new(component_type: MiniTypeId, key: NodeKey) -> Self {
        Self {
            component_type,
            key,
            _marker: PhantomData,
        }
    }
}

impl<T> From<&ComponentId<T>> for MiniTypeId {
    fn from(component_id: &ComponentId<T>) -> Self {
        component_id.component_type
    }
}

impl<T> From<&ComponentId<T>> for NodeKey {
    fn from(component_id: &ComponentId<T>) -> Self {
        component_id.key
    }
}
