#[cfg(test)]
mod tests {
    use necs::{node, Node, NodeTrait, World};

    #[derive(Debug)]
    struct Useless;

    #[node]
    struct Foo<T: 'static + Send + Sync> {
        pub x: Useless,
        y: i32,
        z: i32,
        #[ext]
        bar: T,
    }

    #[node]
    struct Bar {}

    #[node]
    struct Baz;

    trait Process: NodeTrait {
        fn process(&self);
    }

    impl<T: Send + Sync> Process for Foo<'_, T> {
        fn process(&self) {
            println!("{:?}", &self.y);
        }
    }

    #[test]
    fn register_spawn_retrieve() {
        let mut world = World::new();
        world.register_node::<Foo<u32>>();
        world.register_trait::<Foo<u32>, dyn Process, _>(|x| Box::new(x));

        let node_id = world.spawn_node(FooBuilder {
            x: Useless,
            y: 3,
            z: 2,
            bar: 2u32,
        });

        // The node can be retrieved as a concrete type.
        let node: Foo<u32> = world.get_node::<Foo<u32>>(node_id);
        println!("node.x: {:?} node.bar: {}", node.x, node.bar);
        println!("node.process():");
        node.process();
        drop(node); // It also automatically drops when it goes out of scope.
        // Or it may be retrieved as any one of the registered traits (in this
        // case only Process).
        let mut node = world.get_node_resilient::<dyn Process>(node_id);
        node.process();
        // And we can access fields based on their names (or just create a
        // getter and setter on the Process trait instead, but
        // registering traits is boring and quite a lot of work, so this
        // is an alternative).
        println!("Process bar: {}", node.get("bar").to::<u32>());
        drop(node);
        // The Node trait is registered for all nodes automatically.
        let mut node = world.get_node_resilient::<dyn Node>(node_id);
        // And we can access fields with get.
        println!("Node bar: {}", node.get("bar").to::<u32>());
        drop(node);
        for foo in world.get_nodes::<Foo<u32>>() {
            *foo.y = 1;
            println!("Found a Foo");
        }
    }

    mod flamegraph_test {
        use necs::node;

        #[node]
        pub struct Foo {
            pub a: u32,
            pub b: u32,
            #[ext]
            pub c: u32,
        }
    }
}
