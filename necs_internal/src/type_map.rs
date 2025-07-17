use crate::node::NodeId;
use crate::storage::Storage;
use std::any::{Any, TypeId};
use std::collections::BTreeMap;
use crate::NodeRef;

pub fn type_from_id<'a, T: 'static + NodeRef>(
    storage: &'a mut Storage,
    id: NodeId,
    func: &dyn Fn(),
) -> Box<dyn Any> {
    Box::new(unsafe { T::__build_from_storage(storage, id) })
}

#[derive(Debug)]
pub struct TypeMap {
    map: BTreeMap<TypeId, fn(&mut Storage, NodeId, &dyn Fn()) -> Box<dyn Any>>,
}

impl TypeMap {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn register<T: 'static + NodeRef>(&mut self) {
        self.map.insert(TypeId::of::<T::RecipeTuple>(), type_from_id::<T>);
    }

    /// Fetches a node based on only [NodeId].
    /// # Safety
    /// The node associated with the given [NodeId] must be of type [T].
    pub unsafe fn get_node<'a, T: 'static>(
        &self,
        storage: &'a mut Storage,
        id: NodeId,
    ) -> T {
        *self.map[&id.node_type](storage, id, &|| {

        })
    }
}
