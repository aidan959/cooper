use crate::SystemParameter;

use super::{Get, GetError, GetItem, World};

/**
Example System usage  
```
use frost::*;
struct A (i32);
struct B (String);
struct C (A,B);
struct D (C);
let world = World::new();

fn my_system(
    mut query0: Search<(&A, &B)>,
    mut query1: Search<(&C, &D)>,
) {
    for (a, b) in query0.iter() {
        println!("A: {}, B: {}", a.0, b.0);
    }
    for (c, d) in query1.iter() {
        println!("C: ({}, {}), D: ({}, {})", c.0.0, c.1.0, d.0.0.0, d.0.1.0);
    }
}
my_system.run(&world, 123.0).unwrap();
```
*/
pub trait System<P> {
    fn run(self, world: &World, delta_time: f32) -> Result<(), GetError>;
    fn run_fixed(self, world: &World, fixed_update: f32) -> Result<(), GetError>;

}

pub trait IntoSystem<P> {
    fn system(self) -> Box<dyn FnMut(&World, f32) -> Result<(), GetError> + Send + Sync>;
}

pub trait OuterSystem {
    type Input;
    fn run<'world_borrow>(self, world: &'world_borrow World, delta_time: f32) -> Result<(), GetError>;
    fn run_fixed<'world_borrow>(self, world: &'world_borrow World, fixed_update: f32) -> Result<(), GetError>;

}

type InnerItem<'a, 'b, A> =
    <<<A as SystemParameter>::Get as Get<'a>>::Item as GetItem<'b>>::InnerItem;

impl<P, S: System<P> + Sync + Send + 'static + Copy> IntoSystem<P> for S {
    fn system(self,) -> Box<dyn FnMut(&World, f32) -> Result<(), GetError> + Send + Sync> {
        Box::new(move |world, delta_time| self.run(world, delta_time))
    }
}

macro_rules! system_def {
    ($($name: ident),*) => {
        impl<FUNC, $($name: SystemParameter),*> System<($($name,)*)> for FUNC
        where
            FUNC: FnMut($($name,)* f32) + for<'a, 'b> FnMut($(InnerItem<'a, 'b, $name>,)* f32),
        {
            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            fn run<'world_borrow>(mut self, world: &'world_borrow World, delta_time: f32) -> Result<(), GetError> {
                $(let mut $name = $name::Get::get(world)?;)*
                self($($name.inner(),)* delta_time);
                Ok(())
            }
            #[allow(non_snake_case)]
            #[allow(unused_variables)]
            fn run_fixed<'world_borrow>(mut self, world: &'world_borrow World, fixed_update: f32) -> Result<(), GetError> {
                $(let mut $name = $name::Get::get(world)?;)*
                self($($name.inner(),)* fixed_update);
                Ok(())
            }
        }
    };
}

macro_rules! system_defr {
    ($x: ident) => {
        system_def!{$x}
    };

    ($x: ident, $($y: ident),*) => {
        system_def!{$x, $($y),*}
        system_defr!{$($y),*}

    };
}
system_defr!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z);




