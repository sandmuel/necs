#![feature(downcast_unchecked)]

mod register_traits;

pub use necs_internal::World;
pub use necs_internal::{Node, NodeId, NodeRef};

#[cfg(test)]
mod tests {
    use super::*;
    use necs_internal::Node;
    use necs_macros::node;

    #[node]
    struct Foo {
        x: u64,
        y: i32,
        #[ext]
        bar: u32,
    }

    trait Process: Node {
        fn process(&self);
    }

    impl Process for Foo<'_> {
        fn process(&self) {
            println!("{:?}", &self.y);
        }
    }

    #[test]
    fn it_works() {
        let mut world = World::new();
        register_with_traits!(world, Foo<'_>, [Process]);

        let node_id = world.spawn_node(FooBuilder { x: 8, y: 3, bar: 2 });
        // The node can be retrieved as a concrete type.
        let node: Foo = world.get_node::<Foo>(node_id);
        println!("node.x: {} node.bar: {}", node.x, node.bar);
        println!("node.process():");
        node.process();
        // Or it may be retrieved as any one of the registered traits (in this case only
        // Process).
        let mut node: Box<dyn Process> = world.get_node_resilient::<Box<dyn Process>>(node_id);
        node.process();
        // And we can access fields based on their names (or just create a getter and
        // setter on the Process trait instead, but registering traits is boring and
        // quite a lot of work, so this is an alternative).
        println!("{}", node.get("bar").to::<u32>());
        // The #[ext] attribute has these fields stored with others of their type
        // for better use of cache where a single field is often needed (such as
        // transforms). Temporarily made this public, but this is an internal function,
        // and I do not really have any features in this area at the moment.
        for component in world.storage.components.get_all::<u32>() {
            println!("{}", component);
        }
    }
}
