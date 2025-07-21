use crate::NodeRef;
use crate::node::NodeId;
use crate::storage::Storage;
use std::any::{Any, TypeId};
use std::collections::BTreeMap;

pub fn type_from_id<'a, T: 'static + NodeRef>(
    storage: &'a mut Storage,
    id: NodeId,
    _func: &dyn Fn(),
) -> Box<dyn Any + Send> {
    Box::new(unsafe { T::__build_from_storage(storage, id) })
}

pub struct TypeMap {
    map: BTreeMap<TypeId, Box<dyn FnMut(&mut Storage, NodeId, &dyn Fn()) -> Box<dyn Any + Send>>>,
}

impl TypeMap {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn register<T: 'static + NodeRef>(&mut self) {
        self.map.insert(
            TypeId::of::<T::RecipeTuple>(),
            Box::new(|storage, id, _| type_from_id::<T>(storage, id, &|| {})),
        );
    }

    // Register a retriever for trait objects, e.g. Box<dyn Banana>
    pub fn register_trait<T, Trait>(&mut self, f: fn(T) -> Box<Trait>)
    where
        T: NodeRef + 'static,
        Trait: ?Sized + 'static + Send,
    {
        let trait_id = TypeId::of::<Box<Trait>>();
        self.map.insert(
            trait_id,
            Box::new(move |storage, id, _| {
                let node = unsafe { T::__build_from_storage(storage, id) };
                let trait_obj = f(node);
                Box::new(trait_obj) as Box<dyn Any + Send>
            }),
        );
    }

    /// Fetches a node based on only [NodeId].
    /// # Safety
    /// The node associated with the given [NodeId] must be of type [T].
    pub unsafe fn get_node<T: 'static>(&mut self, storage: &mut Storage, id: NodeId) -> T {
        let any = self
            .map
            .get_mut(&TypeId::of::<T>())
            .expect("Type not registered")(storage, id, &|| {});
        *any.downcast::<T>().unwrap()
    }
}
