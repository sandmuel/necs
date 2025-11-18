use crate::node::{Node, NodeId, NodeType};
use crate::storage::Storage;
use crate::{NodeRef, NodeTrait};
use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::mem::transmute;

pub struct TypeMap {
    map: HashMap<
        TypeId,
        HashMap<NodeType, Box<dyn Fn(&Storage, NodeId) -> Box<dyn Any> + Send + Sync>>,
    >,
}

impl TypeMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Registers a type `T` that can be converted to `Trait` using
    /// `to_trait_obj`.
    pub fn register<T, Trait, F>(&mut self, node_type: NodeType, to_trait_obj: F)
    where
        T: NodeRef + Node,
        Trait: NodeTrait + ?Sized + 'static,
        F: Fn(T::Instance<'static>) -> Box<Trait> + Send + Sync + 'static,
    {
        let closure = move |storage: &Storage, id: NodeId| {
            // TODO: ensure get_node() casts back to the proper lifetime.
            let storage: &'static Storage = unsafe { transmute(storage) };
            let node = unsafe { T::__build_from_storage(storage, id) };
            let trait_obj: Box<Trait> = to_trait_obj(node);
            Box::new(trait_obj) as Box<dyn Any>
        };

        self.map
            .entry(TypeId::of::<Trait>())
            .or_insert_with(HashMap::new)
            .insert(node_type, Box::new(closure));
    }

    pub fn get_node<Trait>(&self, storage: &Storage, id: NodeId) -> Box<Trait>
    where
        Trait: 'static + ?Sized,
    {
        let type_map = self
            .map
            .get(&TypeId::of::<Trait>())
            .expect(&format!("trait {} not registered", type_name::<Trait>()));

        let factory = type_map.get(&id.node_type).expect(&format!(
            "type {:?} not registered for Trait {}",
            id.node_type,
            type_name::<Trait>()
        ));

        let trait_obj = factory(storage, id);

        *trait_obj
            .downcast::<Box<Trait>>()
            .expect("Failed to downcast the node to the expected trait object")
    }
}
