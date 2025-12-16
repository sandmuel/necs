# NECS
NECS (Not Entity Component System) is a crate aiming to strike a good balance between ECS patterns and traditional object management.

## Features

### Entities (referred to as nodes) **can** have behavior.
```rust
#[node]
struct MyNode {
    x: u32,
    y: u32,
    transform: Transform,
}

impl MyNode<'_> {
    fn hello() {
        println!("Hello, world!");
        println!("Fields: {}, {}, {}", self.x, self.y, self.transform);
    }
}
```
### Fields can be marked with the `#[ext]` attribute to place them with other fields of the same type as opposed to the object for efficient use of cache.
```rust
#[node]
struct MyNode {
    x: u32,
    y: u32,
    #[ext]
    transform: Transform,
}
```
### Nodes can be retrieved without knowing the concrete type.
```rust
#[node]
struct MyNode {
    x: u32,
    y: u32,
    transform: Transform,
}
world.register_node::<MyNode>();
world.spawn_node(MyNodeBuilder { x: 8, y: 8, transform: Transform::ZERO });
world.get_node_resilient::<dyn Node>(node_id);
```

## Star History

<a href="https://www.star-history.com/#sandmuel/necs&Date">
 <picture>
   <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/svg?repos=sandmuel/necs&type=Date&theme=dark" />
   <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/svg?repos=sandmuel/necs&type=Date" />
   <img alt="Star History Chart" src="https://api.star-history.com/svg?repos=sandmuel/necs&type=Date" />
 </picture>
</a>
