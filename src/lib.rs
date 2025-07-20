#![feature(downcast_unchecked)]

mod register_traits;

pub use necs_internal::World;

#[cfg(test)]
mod tests {
    use super::*;
    use necs_internal::Node;
    use necs_macros::node;

    #[derive(Debug)]
    struct CantCopy;

    #[node]
    struct Foo {
        x: CantCopy,
        y: i32,
        #[ext]
        bar: u32,
    }

    trait Process: Node {
        fn process(&self);
    }

    impl Process for Foo<'_> {
        fn process(&self) {
            println!("{:?}", &self.x);
        }
    }

    #[test]
    fn it_works() {
        let mut world = World::new();
        register_with_traits!(world, Foo<'_>, [Process]);

        let node_id = world.spawn_node(FooBuilder {
            x: CantCopy {},
            y: 3,
            bar: 2,
        });
        let node = world.get_node::<Foo>(node_id);
        println!("{:?}", &node.x);
        println!("{}", node.bar);
        node.process();
        let mut node = world.get_node_resilient::<Box<dyn Process>>(node_id);
        println!("{}", node.get("bar").try_as::<u32>());
        node.process();
    }
}
