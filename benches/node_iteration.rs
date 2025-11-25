#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]

use necs_internal::*;
use necs_macros::node;
use std::any::{TypeId, type_name};
use std::cell::SyncUnsafeCell;

use criterion::{Criterion, criterion_group, criterion_main};
use necs_internal::storage::MiniTypeMap;
use slotmap::SlotMap;

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
                for _ in 0..1_000_000 {
                    world.spawn_node(FooBuilder { a: 1, b: 2, c: 3 });
                }
                world
            },
            // code being measured
            |mut world| {
                world.get_nodes::<Foo>();
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

    let idx = 2u16;
    let a_vec = vec![1, 2, 3];

    c.bench_function("node_iteration", |b| {
        b.iter(|| unsafe {
            let _ = &TypeId::of::<u64>();
            a_vec.get(idx as usize).unwrap_or_else(|| {
                panic!(
                    "cannot get MiniTypeId for unregistered type {:?}",
                    type_name::<u64>()
                )
            })
            //mini_type_map.get_unchecked::<u64, _>(id, key);
            //for node in mini_type_map.values::<u64, _>() {
            //    let _ = node;
            //}
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
