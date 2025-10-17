use slotmap::SlotMap;
use crate::storage::component_storage::ComponentStorage;
use crate::storage::key::NodeKey;
use crate::storage::node_storage::NodeStorage;

mod component_storage;
pub(crate) mod key;
mod node_storage;

#[derive(Debug)]
pub struct Storage {
    // This map only serves to generate unique keys.
    pub key_factory: SlotMap<NodeKey, ()>,
    pub nodes: NodeStorage,
    pub components: ComponentStorage,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            key_factory: SlotMap::with_key(),
            nodes: NodeStorage::new(),
            components: ComponentStorage::new(),
        }
    }
}
