use crate::Storage;
use crate::storage::key::NodeKey;
use std::any::{Any, type_name};
use std::marker::Tuple;

/// A [`u16`] corresponding to a node's type.
pub type NodeType = u16;

/// Used with [`get_node`](crate::World::get_node) or
/// [`get_node_resilient`](crate::World::get_node_resilient) to retrieve nodes
/// stored by [`World`](crate::World).
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct NodeId {
    pub node_type: NodeType,
    pub instance: NodeKey,
}

pub trait Field: Any {}

impl<T: 'static> Field for T {}

impl dyn Field {
    pub fn to<T: 'static>(&mut self) -> &mut T {
        (self as &mut dyn Any)
            .downcast_mut::<T>()
            .unwrap_or_else(|| panic!("invalid downcast to {}", type_name::<T>()))
    }
}

/// Do **not** implement this trait.
/// This trait is only to be implemented by the corresponding proc macro crate.
pub trait NodeBuilder {
    /// The implementation of [`NodeRef`] associated with this implementation of
    /// [`NodeBuilder`].
    type AsNodeRef: NodeRef;

    /// Moves all fields to a given [`Storage`].
    fn __move_to_storage(self, storage: &mut Storage) -> NodeId;
}

/// Do **not** implement this trait.
/// This trait is only to be implemented by the corresponding proc macro crate.
pub trait NodeRef: 'static + NodeTrait + Send + Sync {
    type Instance<'node>: Node;
    type RecipeTuple: Tuple + Send + Sync;

    /// Assembles a [`NodeRef`] from fields stored in the given [`Storage`].
    /// # Safety
    /// The safety of this depends on the key-value pairs always being correct
    /// to ensure the safety of unchecked downcasts.
    unsafe fn __build_from_storage<'node>(
        storage: &'node Storage,
        id: NodeId,
    ) -> Self::Instance<'node>;

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
/// ```
/// use necs::{Node, World, node};
/// let mut world = World::new();
///
/// #[node]
/// struct MyNode {
///     my_field: u32,
///     other_field: i32,
/// }
///
/// // Register our node.
/// world.register_node::<MyNode>();
///
/// // Spawn our node.
/// let node_id = world.spawn_node(MyNodeBuilder { my_field: 8, other_field: 0 });
///
/// // Now we can retrieve it as a dyn Node.
/// let mut node = world.get_node_resilient::<dyn Node>(node_id);
///
/// // And we can access fields using get() since Node is a subtrait of NodeTrait.
/// println!("my_field: {}", node.get("my_field").to::<u32>());
/// ```
pub trait Node: NodeTrait {}

impl<T> Node for T where T: NodeTrait {}
