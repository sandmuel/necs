use slotmap::new_key_type;

new_key_type! {
    /// A key used to uniquely identify a node's tuple and components in
    /// [`World`].
    ///
    /// This key is generally used with the [`NodeId`] or the [`ComponentId`]
    /// wrapper, but may be used individually as a unique identifier for node
    /// metadata.
    ///
    /// [`World`]: crate::World
    /// [`NodeId`]: crate::NodeId
    /// [`ComponentId`]: crate::ComponentId
    pub struct ItemKey;
}
