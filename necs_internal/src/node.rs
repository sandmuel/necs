use crate::Storage;
use std::any::{Any, TypeId, type_name};
use std::marker::Tuple;
use crate::storage::map_key::MapKey;

/// Used with [`get_node`](crate::World::get_node) or
/// [`get_node_resilient`](crate::World::get_node_resilient) to retrieve nodes
/// stored by [`World`](crate::World).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NodeId {
    pub node_type: TypeId,
    pub instance: MapKey,
}

pub trait Field: Any {}

impl<T: 'static + Any> Field for T {}

impl<'a> dyn Field {
    pub fn to<T: 'static>(&'a mut self) -> &'a mut T {
        (self as &mut dyn Any)
            .downcast_mut::<T>()
            .expect(&format!("invalid downcast to {}", type_name::<T>()))
    }
}

/// Do **not** implement this trait.
/// This trait is only to be implemented by the corresponding proc macro crate.
pub trait NodeBuilder {
    /// The implementation of [`NodeRef`] associated with this implementation of
    /// [`NodeBuilder`].
    type AsNodeRef: 'static + NodeRef;

    /// Moves all fields to a given [`Storage`].
    /// # Safety
    /// Do *anything* wrong, and you could cause panics, undefined behavior, and
    /// more! Good luck.
    unsafe fn __move_to_storage(self, storage: &mut Storage) -> NodeId;
}

/// Do **not** implement this trait.
/// This trait is only to be implemented by the corresponding proc macro crate.
pub trait NodeRef: NodeTrait + Send + Sync {
    type RecipeTuple: Tuple;

    /// Assembles a [`NodeRef`] from fields stored in the given [`Storage`].
    /// # Safety
    /// Do *anything* wrong, and you could cause panics, undefined behavior, and
    /// more! Good luck.
    unsafe fn __build_from_storage(storage: &mut Storage, id: NodeId) -> Self;

    /// Registers this node to node storage and all fields with the `#[ext]`
    /// attribute to component storage.
    fn __register_node(storage: &mut Storage);
}

/// Require this on any trait that should be compatible with
/// [`get_node_resilient`](crate::World::get_node_resilient).
pub trait NodeTrait: Send + Sync {
    fn get(&mut self, field_name: &str) -> &mut dyn Field;
}

/// A subtrait of [`NodeTrait`] that is automatically implemented and registered
/// for every node.
/// ```ignore
/// #[node]
/// struct MyNode {
///     my_field: u32,
/// }
///
/// // Register our node.
/// world.register_node::<MyNode>();
///
/// // Now we can retrieve it as a dyn Node.
/// let mut node = world.get_node_resilient::<dyn Node>();
///
/// // And we can access fields using get() since Node is a subtrait of NodeTrait.
/// println!("my_field: {}", node.get("my_field").to::<u32>());
/// ```
pub trait Node: NodeTrait {}

impl<T> Node for T where T: NodeTrait {}
