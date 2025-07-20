/// Macro to register a node and any number of trait retrievers for it.
/// Usage:
///   register_node_with_traits!(world, Foo<'_>, [Banana, AnotherTrait]);
#[macro_export]
macro_rules! register_with_traits {
    ($world:expr, $ty:ty, [$($trait:ident),* $(,)?]) => {{
        $world.register_node::<$ty>();
        $(
            $world.type_map.register_trait::<$ty, dyn $trait>(|node| Box::new(node) as Box<dyn $trait>);
        )*
    }};
}
