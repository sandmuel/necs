use necs_internal::*;
use necs_macros::node;

use criterion::{Criterion, criterion_group, criterion_main};

#[node]
struct Foo {
    a: u32,
    b: u32,
    c: u32,
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut world = World::new();
    world.register_node::<Foo>();
    for _ in 0..1_000_000 {
        world.spawn_node(FooBuilder { a: 1, b: 2, c: 3 });
    }
    let id = world.spawn_node(FooBuilder { a: 1, b: 2, c: 3 });
    c.bench_function("node_iteration", |b| b.iter(|| {
        for node in world.get_nodes::<Foo>() {
        }
    }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
