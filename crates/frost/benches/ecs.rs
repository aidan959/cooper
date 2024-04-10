use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use frost::*;
struct SampleStruct {
    a: i32,
    b: f32,
    c: i64,
}
struct SampleStructB {
    a: i32,
    b: f32,
    c: i64,
    d: i64,

}struct SampleStructC {
    a: i32,
    b: f32,
    c: i64,
    d: u64,
}
fn world_system(mut search: Search<(&SampleStruct,)>, _delta_time: f32){
    for i in search.iter() {
    }
}
fn world_insertions(amount: usize){
    let mut world = World::new();
    for i in 0..amount {
        world.new_entity((SampleStruct{a:i as i32,b:i as f32,c:i as i64},)).unwrap();
    }
}
fn world_insertions_diverse(amount: usize){
    let mut world = World::new();
    insertions_diverse(&mut world, amount)
}
fn insertions_diverse(world: &mut World, amount: usize){

    for i in 0..amount {
        world.new_entity((SampleStruct{a:i as i32,b:i as f32,c:i as i64},SampleStructB{a:i as i32,b:i as f32,c:i as i64,d:1 as i64},)).unwrap();
        world.new_entity((SampleStruct{a:i as i32,b:i as f32,c:i as i64},SampleStructB{a:i as i32,b:i as f32,c:i as i64,d:1 as i64},
                        SampleStructC{a:i as i32,b:i as f32,c:i as i64,d:1 as u64})).unwrap();
        world.new_entity((SampleStruct{a:i as i32,b:i as f32,c:i as i64}, SampleStructC{a:i as i32,b:i as f32,c:i as i64,d:1 as u64},)).unwrap();
    }
}
fn insertions(world: &mut World, amount: usize){
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
    let sizes = [10,20,50,100,200,400,500,1000,1500,2000,5000,10000,20000,50000,100000,200000,500000,1000000];    
    for size in sizes {
        c.bench_with_input(BenchmarkId::new("Insert New Entity", size), &size, |b, &s| {
            b.iter(||{world_insertions(s)});
        });
        let mut world = World::new();
        insertions(&mut world, size);
        c.bench_with_input(BenchmarkId::new("Search Entity", size), &size, |b, &s| {
            b.iter(||{world_search.run(&world, 0.0)});
        });

        c.bench_with_input(BenchmarkId::new("Insert New Entity (diverse)", size), &size, |b, &s| {
            b.iter(||{world_insertions_diverse(s)});
        });
        let mut world = World::new();
        insertions_diverse(&mut world, size);
        c.bench_with_input(BenchmarkId::new("Search Entity (diverse)", size), &size, |b, &s| {
            b.iter(||{world_search.run(&world, 0.0)});
        });
        
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);