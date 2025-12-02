mod component_storage;
mod mini_type_map;
mod node_storage;

pub use crate::storage::mini_type_map::NodeKey;
pub(crate) use component_storage::ComponentStorage;
pub use mini_type_map::MiniTypeId;
pub use mini_type_map::MiniTypeMap;
pub use mini_type_map::MiniTypeMapKey;
pub use node_storage::BorrowDropper;
pub(crate) use node_storage::NodeStorage;

// TODO: Merge this with World if no cache impact.
#[derive(Debug)]
pub struct Storage {
    pub nodes: NodeStorage,
    pub components: ComponentStorage,
}

impl Storage {
    #[inline(always)]
    pub fn new() -> Self {
        Storage::default()
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            nodes: NodeStorage::new(),
            components: ComponentStorage::new(),
        }
    }
}
