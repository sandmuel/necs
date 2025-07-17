#![feature(downcast_unchecked)]

pub use necs_internal::World;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use necs_internal::ComponentId;
    use necs_macros::node;
    use std::any::{Any, TypeId};

    #[node]
    struct Foo {
        x: u64,
        y: i32,
        #[ext]
        bar: u32,
    }

    trait Banana {
        fn banana(&self);
    }

    impl Banana for Foo<'_> {
        fn banana(&self) {
            println!("{}", self.x);
        }
    }

    #[test]
    fn it_works() {
        let mut world = World::new();

        world.register_node::<Foo>();

        let node_id = world.spawn_node(FooBuilder { x: 8, y: 3, bar: 2 });
        println!("node_id = {:?}", node_id);
        println!("{:#?}", world);
        let node = world.get_node::<Foo>(node_id);
        println!(
            "{:?}, {:?}",
            node_id.node_type,
            TypeId::of::<(u64, i32, ComponentId<u32>)>()
        );
        let node = world.get_node_resilient::<Box<dyn Banana>>(node_id);
        node.banana();
    }
}
