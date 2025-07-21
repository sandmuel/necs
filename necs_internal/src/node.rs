use crate::Storage;
use slotmap::DefaultKey;
use std::any::{Any, TypeId, type_name};
use std::marker::Tuple;

#[derive(Debug, Copy, Clone)]
pub struct NodeId {
    pub node_type: TypeId,
    pub instance: DefaultKey,
}

pub trait Field: Any {}

impl<T: 'static + Any> Field for T {}

impl dyn Field {
    pub fn to<T: 'static>(&mut self) -> &mut T {
        (self as &mut dyn Any)
            .downcast_mut::<T>()
            .expect(&format!("invalid downcast to {}", type_name::<T>()))
    }
}

/// Do **not** implement this trait.
/// This trait is only to be implemented by the corresponding proc macro crate.
pub trait NodeBuilder {
    /// The implementation of [NodeRef] associated with this implementation of
    /// [NodeBuilder].
    type AsNodeRef: 'static + NodeRef;

    /// Moves all fields to a given [Storage].
    /// # Safety
    /// Do *anything* wrong, and you could cause panics, undefined behavior, and
    /// more! Good luck.
    unsafe fn __move_to_storage(self, storage: &mut Storage) -> NodeId;
}

/// Do **not** implement this trait.
/// This trait is only to be implemented by the corresponding proc macro crate.
pub trait NodeRef: Send {
    type RecipeTuple: Tuple;

    /// Assembles a [NodeRef] from fields stored in the given [Storage].
    /// # Safety
    /// Do *anything* wrong, and you could cause panics, undefined behavior, and
    /// more! Good luck.
    unsafe fn __build_from_storage(storage: &mut Storage, id: NodeId) -> Self;

    /// Registers this node to node storage and all fields with the `#[ext]`
    /// attribute to component storage.
    fn __register_node(storage: &mut Storage);
}

pub trait Node {
    fn get(&mut self, field_name: &str) -> &mut dyn Field;
}
