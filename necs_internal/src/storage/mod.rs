use crate::storage::component_storage::ComponentStorage;
use crate::storage::node_storage::NodeStorage;

mod component_storage;
mod node_storage;

#[derive(Debug)]
pub struct Storage {
    pub nodes: NodeStorage,
    pub components: ComponentStorage,
}

impl Storage {
    pub fn new() -> Self {
        Self {
            nodes: NodeStorage::new(),
            components: ComponentStorage::new(),
        }
    }
}
