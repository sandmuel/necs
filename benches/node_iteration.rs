#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]
#![feature(downcast_unchecked)]

use criterion::{Criterion, criterion_group, criterion_main};
use necs_internal::*;
use necs_macros::node;
use std::hint::black_box;

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

    c.bench_function("node_iteration", |b| {
        b.iter(|| {
            black_box(world.get_node::<Foo>(id));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
