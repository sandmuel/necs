use slotmap::new_key_type;

new_key_type! {
    /// A key used to uniquely identify a node's tuple and components in [`World`]. This key is always used with the [`NodeId`] or the [`ComponentId`] wrapper.
    ///
    /// [`World`]: crate::World
    /// [`NodeId`]: crate::NodeId
    /// [`ComponentId`]: crate::ComponentId
    pub(crate) struct NodeKey;
}
