#![feature(downcast_unchecked)]

#[cfg(test)]
mod tests {
    use necs::{Node, NodeTrait, World, node};

    #[node]
    struct Foo {
        pub x: u64,
        y: i32,
        //#[ext] // TODO: fix multiple ext fields causes double mut borrow.
        z: i32,
        #[ext]
        bar: u32,
    }

    trait Process: NodeTrait {
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
        world.register_node::<Foo>();
        world.node_map.register::<Foo, dyn Process>(|x| Box::new(x));

        let node_id = world.spawn_node(FooBuilder {
            x: 8,
            y: 3,
            z: 2,
            bar: 2,
        });
        // The node can be retrieved as a concrete type.
        let _node_1: Foo = world.get_node::<Foo>(node_id);
        let node: Foo = world.get_node::<Foo>(node_id);
        println!("node.x: {} node.bar: {}", node.x, node.bar);
        println!("node.process():");
        node.process();
        // Or it may be retrieved as any one of the registered traits (in this case only
        // Process).
        let mut node = world.get_node_resilient::<dyn Process>(node_id);
        node.process();
        // And we can access fields based on their names (or just create a getter and
        // setter on the Process trait instead, but registering traits is boring and
        // quite a lot of work, so this is an alternative).
        println!("Process bar: {}", node.get("bar").to::<u32>());
        // Node trait is registered for all nodes automatically.
        let mut node = world.get_node_resilient::<dyn Node>(node_id);
        // And we can access fields with get.
        println!("Node bar: {}", node.get("bar").to::<u32>());
        for _ in world.get_nodes::<Foo>() {
            println!("Found a Foo");
        }
    }
}
