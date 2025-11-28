mod component_storage;
mod mini_type_map;
mod node_storage;

use crate::NodeKey;
pub(crate) use component_storage::ComponentStorage;
pub use mini_type_map::MiniTypeMap;
pub use node_storage::BorrowDropper;
pub(crate) use node_storage::NodeStorage;
use slotmap::SlotMap;

// TODO: Merge this with World if no cache impact.
//#[derive(Debug)]
pub struct Storage {
    // This map only serves to generate unique keys.
    pub(crate) key_factory: SlotMap<NodeKey, ()>,
    pub nodes: NodeStorage,
    pub components: ComponentStorage,
}

impl Storage {
    #[inline(always)]
    pub fn new() -> Self {
        Storage::default()
    }

    #[inline(always)]
    pub fn mint_key(&mut self) -> NodeKey {
        self.key_factory.insert(())
    }
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            key_factory: SlotMap::with_key(),
            nodes: NodeStorage::new(),
            components: ComponentStorage::new(),
        }
    }
}
