use crate::NodeRef;
use crate::node::{Node, NodeId};
use crate::storage::Storage;
use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;

pub struct TypeMap {
    map: HashMap<
        TypeId,
        HashMap<TypeId, Box<dyn Fn(&mut Storage, NodeId) -> Box<dyn Any + Send + Sync> + Send + Sync>>,
    >,
}

impl TypeMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    /// Registers a type `T` that can be converted to `Trait` using `to_trait_obj`.
    pub fn register<T, Trait>(&mut self, to_trait_obj: fn(T) -> Box<Trait>)
    where
        T: 'static + NodeRef + Node,
        Trait: 'static + Send + Sync + ?Sized,
    {
        let closure = move |storage: &mut Storage, id: NodeId| {
            let node = unsafe { T::__build_from_storage(storage, id) };
            let trait_obj = to_trait_obj(node);
            Box::new(trait_obj) as Box<dyn Any + Send + Sync>
        };

        self.map
            .entry(TypeId::of::<Trait>())
            .or_insert_with(HashMap::new)
            .insert(TypeId::of::<T::RecipeTuple>(), Box::new(closure));
    }

    pub fn get_node<Trait>(&mut self, storage: &mut Storage, id: NodeId) -> Box<Trait>
    where
        Trait: 'static + Send + Sync + ?Sized,
    {
        let type_map = self
            .map
            .get(&TypeId::of::<Trait>())
            .expect(&format!("Trait {} not registered", type_name::<Trait>()));

        let factory = type_map
            .get(&id.node_type)
            .expect(&format!("Type {:?} not registered for Trait {}", id.node_type, type_name::<Trait>()));

        let trait_obj = factory(storage, id);

        // It's a Box<Box<Trait>> — so we unwrap it
        *trait_obj
            .downcast::<Box<Trait>>()
            .expect("Failed to downcast node to expected trait object")
    }
}
