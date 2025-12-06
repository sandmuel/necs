use crate::node::{Node, NodeId};
use crate::storage::{MiniTypeId, Storage};
use crate::{NodeRef, NodeTrait};
use rustc_hash::FxHashMap as HashMap;
use std::any::{Any, TypeId, type_name};
use std::fmt::{Debug, Formatter};
use std::mem::transmute;

pub struct TraitMap {
    map: HashMap<
        TypeId,
        HashMap<MiniTypeId, Box<dyn Fn(&Storage, NodeId) -> Box<dyn Any> + Send + Sync>>,
    >,
    trait_names: HashMap<TypeId, &'static str>,
    node_names: HashMap<MiniTypeId, &'static str>,
}

impl Debug for TraitMap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let trait_type_keys = self.map.keys();
        if trait_type_keys.len() == 0 {
            return write!(f, "{{}}");
        }
        write!(f, "{{")?;
        if f.alternate() {
            write!(f, "\n")?;
        }
        for (i, trait_type_key) in trait_type_keys.enumerate() {
            if i > 0 {
                if f.alternate() {
                    write!(f, ",\n")?;
                } else {
                    write!(f, ", ")?;
                }
            }
            let trait_name = self.trait_names.get(trait_type_key).unwrap();
            let registered_nodes = self.map[&trait_type_key]
                .keys()
                .map(|x| self.node_names.get(x).unwrap());
            write!(f, "    {}: [", trait_name)?;
            if f.alternate() {
                write!(f, "\n        ")?;
            }
            for (i, node) in registered_nodes.enumerate() {
                if i > 0 {
                    if f.alternate() {
                        write!(f, ",\n        ")?;
                    } else {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "{}", node)?;
            }
            if f.alternate() {
                write!(f, ",\n    ]")?;
            } else {
                write!(f, "]")?;
            }
        }
        if f.alternate() {
            write!(f, ",\n")?;
        }
        write!(f, "}}")
    }
}

impl TraitMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
            trait_names: HashMap::default(),
            node_names: HashMap::default(),
        }
    }

    /// Registers a type `T` that can be converted to `Trait` using
    /// `to_trait_obj`.
    pub fn register<T, Trait, F>(&mut self, node_type: MiniTypeId, to_trait_obj: F)
    where
        T: NodeRef + Node,
        Trait: NodeTrait + ?Sized + 'static,
        F: Fn(T::Instance<'static>) -> Box<Trait> + Send + Sync + 'static,
    {
        let closure = move |storage: &Storage, id: NodeId| {
            // TODO: ensure get_node() casts back to the proper lifetime.
            let storage: &'static Storage = unsafe { transmute(storage) };
            let (recipe_tuple, borrow_dropper) = storage.nodes.get_element::<T>(id);
            let node =
                unsafe { T::__build_from_storage(recipe_tuple, borrow_dropper, storage, id) };
            let trait_obj: Box<Trait> = to_trait_obj(node);
            Box::new(trait_obj) as Box<dyn Any>
        };

        self.map
            .entry(TypeId::of::<Trait>())
            .or_default()
            .insert(node_type, Box::new(closure));

        self.trait_names
            .entry(TypeId::of::<Trait>())
            .or_insert(type_name::<Trait>());
        self.node_names.entry(node_type).or_insert(type_name::<T>());
    }

    pub fn get_node<Trait>(&self, storage: &Storage, id: NodeId) -> Box<Trait>
    where
        Trait: 'static + ?Sized,
    {
        let type_map = self
            .map
            .get(&TypeId::of::<Trait>())
            .unwrap_or_else(|| panic!("trait {} not registered", type_name::<Trait>()));

        let factory = type_map.get(&id.node_type).unwrap_or_else(|| {
            panic!(
                "type {:?} not registered for Trait {}",
                id.node_type,
                type_name::<Trait>()
            )
        });

        let trait_obj = factory(storage, id);

        *trait_obj
            .downcast::<Box<Trait>>()
            .expect("Failed to downcast the node to the expected trait object")
    }
}
