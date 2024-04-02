use crate::SysParam;

use super::{Retrieve, RetrieveError, RetrieveItem, World};

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
    fn run(&mut self, world: &World, delta_time: f32) -> Result<(), RetrieveError>;
    fn run_fixed(&mut self, world: &World, fixed_update: f32) -> Result<(), RetrieveError>;

}

pub trait IntoSystem<P> {
    fn system(self) -> Box<dyn FnMut(&World, f32) -> Result<(), RetrieveError> + Send + Sync>;
}

pub trait OuterSystem {
    type Input;
    fn run<'world>(self, world: &'world World, delta_time: f32) -> Result<(), RetrieveError>;
    fn run_fixed<'world>(self, world: &'world World, fixed_update: f32) -> Result<(), RetrieveError>;

}

type InnerComponent<'a, 'b, T> = <<<T as SysParam>::Retrieve as Retrieve<'a>>::Item as RetrieveItem<'b>>::InnerComponent;

impl<P, S> IntoSystem<P> for S where S: System<P> + Sync + Send + 'static + Copy {
    #[inline]
    fn system(mut self,) -> Box<dyn FnMut(&World, f32) -> Result<(), RetrieveError> + Send + Sync> {
        Box::new(move |world, delta_time| self.run(world, delta_time))
    }
}

macro_rules! system_def {
    ($($name: ident),*) => {
        impl<FUNC, $($name: SysParam),*> System<($($name,)*)> for FUNC
        where
            FUNC: FnMut($($name,)* f32) + for<'a, 'b> FnMut($(InnerComponent<'a, 'b, $name>,)* f32),
        {
            fn run<'world>(&mut self, world: &'world World, delta_time: f32) -> Result<(), RetrieveError> {
                self($($name::Retrieve::retrieve(world)?.inner(),)* delta_time);
                Ok(())
            }
            fn run_fixed<'world>(&mut self, world: &'world World, fixed_update: f32) -> Result<(), RetrieveError> {
                self($($name::Retrieve::retrieve(world)?.inner(),)* fixed_update);
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


system_defr!{
    A,B,C,D,E,F,G,H,I,J,K,L,M,N,O,P,Q,R,S,T,U,V,W,X,Y,Z
}
/* 
impl <FUNC,ABC:SysParam>System<(ABC,)>for FUNC where FUNC:FnMut(ABC,f32)+for<'a,'b>FnMut(InnerComponent<'a,'b,ABC>,f32){
    #[allow(non_snake_case)]
    fn run<'world>(&mut self,world: &'world World,delta_time:f32) -> Result<(),RetrieveError>{
        let mut ABC = ABC::Retrieve::retrieve(world)?;
        self(ABC.inner(), delta_time);
        Ok(())
    }
    #[allow(non_snake_case)]
    fn run_fixed<'world>(&mut self,world: &'world World,fixed_update:f32) -> Result<(),RetrieveError>{
        let mut ABC = ABC::Retrieve::retrieve(world)?;

        self(ABC.inner(),fixed_update);
        Ok(())
    }
} */
