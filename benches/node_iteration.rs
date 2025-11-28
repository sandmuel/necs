#![feature(sync_unsafe_cell)]
#![feature(ptr_as_ref_unchecked)]
#![feature(downcast_unchecked)]

use criterion::{Criterion, criterion_group, criterion_main};
use necs_internal::storage::{MiniTypeMap, UselessWrapper};
use necs_internal::*;
use necs_macros::node;
use slotmap::{SlotMap, SparseSecondaryMap};
use std::any::{Any, type_name};
use std::cell::SyncUnsafeCell;
use std::hint::black_box;
use std::marker::PhantomData;

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

    let useless_wrapper = UselessWrapper::new(SparseSecondaryMap::<NodeKey, u64>::default());

    c.bench_function("node_iteration", |b| {
        b.iter(|| unsafe {
            black_box(mini_type_map.data.get(id.index())); // Takes ~0.2 ns
            black_box(useless_wrapper.data.get(id.index())); // Takes ~0.2 ns
            black_box(mysterious_8::<u64>(&mini_type_map.data, id, key)); // Takes ~9.5 ns
            black_box(mysterious_8::<u64>(&useless_wrapper.data, id, key)); // Takes ~1.5 ns
        })
    });
}

#[inline(never)]
fn mysterious_8<T: 'static + Send + Sync>(
    vec: &Vec<Box<dyn Any + Send + Sync>>,
    id: MiniTypeId,
    key: NodeKey,
) -> Option<&T> {
    let sub_map = unsafe {
        vec.get(id.index())
            .unwrap_or_else(|| {
                panic!(
                    "cannot get MiniTypeId for unregistered type {:?}",
                    type_name::<T>()
                )
            })
            // As long as this function's invariant is upheld, this is safe.
            .downcast_unchecked_ref::<SparseSecondaryMap<NodeKey, T>>()
    };
    sub_map.get(key)
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
