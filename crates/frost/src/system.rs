use crate::SysParam;

use super::{Retrieve, RetrieveError, RetrieveItem, World};

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