use std::time::Instant;
use std::collections::HashMap;
use winit::{KeyboardInput, ScanCode, ElementState};


pub struct KeyInput {
    pub instant: Option<Instant>,
    pub value : f32,
}
pub struct InputManager {
    pub inputs : HashMap<ScanCode, KeyInput>,
}
impl InputManager {

    pub fn key_event(&mut self, k: KeyboardInput) {
        match k.state {
            ElementState::Pressed => {
                match self.inputs.get_mut(&k.scancode) {
                    None=>return,
                    Some (key)=>{ key.value = 1. } ,
                };        
            } 
            ElementState::Released => {
                match self.inputs.get_mut(&k.scancode) {
                    None=>return,
                    Some (key)=>{ key.value = 0. } , };
            }

        }   
    }
}