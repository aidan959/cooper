use criterion::{black_box, criterion_group, criterion_main, Criterion};
use frost::*;
struct SampleStruct {
    a: i32,
    b: f32,
    c: i64,
}
fn world_system(mut search: Search<(&SampleStruct,)>, _delta_time: f32){
    for i in search.iter() {
    }
}
fn world_insertions(world: &mut World, amount: usize){
    for i in 0..amount {
        world.new_entity((SampleStruct{a:i as i32,b:i as f32,c:i as i64},)).unwrap();
    }
}

fn world_search(mut search: Search<(&SampleStruct,)>, _delta_time: f32){
    for _ in search.iter() {
    }
}
fn world_search_mut(mut search: Search<(&mut SampleStruct,)>, _delta_time: f32){
    for i in search.iter() {
        i.a += 1;   
    }
}
fn criterion_benchmark(c: &mut Criterion) {
    let mut world = &mut World::new();
    c.bench_function("world insert 10", |b| b.iter(|| world_insertions(black_box(&mut world), black_box(10))));
    c.bench_function("world search iter 10", |b| b.iter(|| world_system.run(black_box(&mut world), 0.0) ));
    c.bench_function("world search iter mut 10", |b| b.iter(|| world_search_mut.run(black_box(&mut world), 0.0) ));
    
    let mut world = &mut World::new();
    c.bench_function("world insert 100", |b| b.iter(|| world_insertions(black_box(&mut world), black_box(100))));
    c.bench_function("world search iter 100", |b| b.iter(|| world_system.run(black_box(&mut world), 0.0) ));
    c.bench_function("world search iter mut 100", |b| b.iter(|| world_search_mut.run(black_box(&mut world), 0.0) ));
    
    let mut world = &mut World::new();
    c.bench_function("world insert 1000", |b| b.iter(|| world_insertions(black_box(&mut world), black_box(1000))));
    c.bench_function("world search iter 1000", |b| b.iter(|| world_system.run(black_box(&mut world), 0.0) ));
    c.bench_function("world search iter mut 1000", |b| b.iter(|| world_search_mut.run(black_box(&mut world), 0.0) ));
    
    let mut world = &mut World::new();
    c.bench_function("world insert 10000", |b| b.iter(|| world_insertions(black_box(&mut world), black_box(10000))));
    c.bench_function("world search iter 10000", |b| b.iter(|| world_system.run(black_box(&mut world), 0.0) ));
    c.bench_function("world search iter mut 10000", |b| b.iter(|| world_search_mut.run(black_box(&mut world), 0.0) ));

}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);