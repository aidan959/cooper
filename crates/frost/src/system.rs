use super::{Retrieve, RetrieveItem, World};

pub trait System<P> {
    fn run(&mut self, world: &World, delta_time: f32) -> ();
    fn run_fixed(&mut self, world: &World, fixed_update: f32) -> ();

}

pub trait IntoSystem<P> {
    fn system(self) -> Box<dyn FnMut(&World, f32) -> () + Send + Sync>;
}


type InnerComponent<'a, 'b, T> = <<<T as SysParam>::Retrieve as Retrieve<'a>>::Item as RetrieveItem<'b>>::InnerComponent;

