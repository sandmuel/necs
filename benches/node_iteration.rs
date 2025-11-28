#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]
#![feature(downcast_unchecked)]

use criterion::{Criterion, criterion_group, criterion_main};
use necs_internal::storage::MiniTypeMap;
use necs_internal::*;
use necs_macros::node;
use slotmap::{SlotMap, SparseSecondaryMap};
use std::any::{Any, type_name};
use std::cell::SyncUnsafeCell;
use rustc_hash::FxHashMap as HashMap;
use std::hint::black_box;

#[node]
struct Foo {
    a: u32,
    b: u32,
    c: u32,
}

fn criterion_benchmark(c: &mut Criterion) {
    /*
    c.bench_function("node_iteration", |b| {
        b.iter_batched(
            // setup code (runs once per sample)
            || {
                let mut world = World::new();
                world.register_node::<Foo>();
                for _ in 0..1 {
                    world.spawn_node(FooBuilder { a: 1, b: 2, c: 3 });
                }
                let id = world.spawn_node(FooBuilder { a: 1, b: 2, c: 3 });
                (world, id)
            },
            // code being measured
            |(mut world, id)| {
                world.get_node::<Foo>(id);
            },
            criterion::BatchSize::LargeInput,
        );
    });

     */

    let mut key_mint = SlotMap::<NodeKey, ()>::default();
    let mut mini_type_map = MiniTypeMap::default();
    mini_type_map.register::<u64, _>();
    let mut key = key_mint.insert(());
    mini_type_map.insert::<u64, _>(key, SyncUnsafeCell::from(8u64));
    let id = mini_type_map.mini_type_of::<u64>();

    let mut internal_map = HashMap::<NodeKey, u64>::default();
    internal_map.insert(key, 8u64);

    let mut a_vec = Vec::<Box<dyn Any>>::default();
    a_vec.push(Box::new(internal_map));

    let mut world = World::new();
    world.register_node::<Foo>();
    for _ in 0..1 {
        world.spawn_node(FooBuilder { a: 1, b: 2, c: 3 });
    }
    let id = world.spawn_node(FooBuilder { a: 1, b: 2, c: 3 });

    c.bench_function("node_iteration", |b| {
        b.iter(|| unsafe {
            black_box(world.get_node::<Foo>(id));
            /*
            let sub_map = unsafe {
                a_vec
                    .get(id.index())
                    .unwrap_or_else(|| panic!("cannot get MiniTypeId for unregistered type u64"))
                    // As long as this function's invariant is upheld, this is safe.
                    .downcast_unchecked_ref::<HashMap<NodeKey, u64>>()
            };

             */
            //black_box(sub_map.get(&key));
            //black_box(mini_type_map.mini_type_of::<u64>()); // Takes ~0.2 ns
            //black_box(mini_type_map.get_unchecked::<u64, _>(id, key));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
