use crate::ItemKey;

#[derive(Debug)]
pub struct Relations {
    parent: Option<ItemKey>,
    children: Vec<ItemKey>,
}

impl Relations {
    pub fn new(parent: Option<ItemKey>) -> Self {
        Self {
            parent,
            children: Vec::default(),
        }
    }
}
