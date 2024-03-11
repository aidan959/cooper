use std::sync::mpsc::Sender;

use crate::application::{GameEvent};
pub trait GameCallbacks <A,B,T>
where
    Self: Sized,
    A: FnMut(Self),
    B: FnMut(Self,Sender<GameEvent>, f32),
    T: FnMut(Self,Sender<GameEvent> , f32),
{
    fn on_start(self, event_sender: Sender<GameEvent>, delta:f32);
    fn update(self, event_sender: Sender<GameEvent>, delta:f32);
    fn fixed_update(self, event_sender: Sender<GameEvent>, delta:f32);
}
pub struct EngineCallbacks<A,B,T>
where
    A: FnMut(),
    B: FnMut(Sender<GameEvent>, f32),
    T: FnMut(Sender<GameEvent> , f32),

{
    pub on_start: A,
    pub update: T,
    pub fixed_update: B
}