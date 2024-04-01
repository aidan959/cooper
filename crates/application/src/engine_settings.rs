use std::{time::Duration, fmt::Display};
use log::{debug,error,info,warn};
use std::fmt::Debug;

use crate::application::WindowSize;
pub const DEFAULT_UPDATE_RATE : f64 = 64.0;
pub const DEFAULT_MAX_FPS : u16 = 128;
pub const DEFAULT_FPS_CAP : bool = true;
pub const DEFAULT_MAX_INTERVAL : f64 = 1.0 / DEFAULT_MAX_FPS as f64;
pub struct FPSSettings {
    pub frame_time : Duration,
    pub limit : bool,
}
impl FPSSettings {
    pub fn max_fps(&self) -> f64{
        1.0 /  self.frame_time.as_secs_f64()
    }
    pub fn set_max_fps(mut self, max_fps : f64) {
        self.frame_time = interval_from_frequency(max_fps);
        self.limit = true;
    }
}
pub struct FPSSettingsBuilder {
    pub frame_time : Duration,
    pub limit : bool,
}
impl FPSSettingsBuilder {
    
}
pub struct EngineSettings {
    pub fixed_update_rate : Duration,
    pub fps_settings : FPSSettings,
    pub window_name : String,
    pub window_size : WindowSize,
}
pub struct EngineSettingsBuilder {
    pub fixed_update_rate : Duration,
    pub fps_settings : FPSSettings,
    pub window_name : String,
    pub window_size : WindowSize,

}
const WIDTH: f64 = 1280.;
const HEIGHT: f64 = 720.;
impl EngineSettingsBuilder {
    pub fn new() -> Self {
        Self {
            fixed_update_rate: interval_from_frequency(DEFAULT_UPDATE_RATE),
            fps_settings: FPSSettings { frame_time: interval_from_frequency(DEFAULT_MAX_FPS), limit: DEFAULT_FPS_CAP },
            window_name: "Cooper".to_string(),
            window_size: (WIDTH, HEIGHT)
        }
    }
    pub fn update_rate_hz(&mut self, frequency: f64) -> &Self {
        self.fixed_update_rate = interval_from_frequency(frequency);
        self
    }
    pub fn fixed_update_rate(&mut self, interval: Duration) -> &Self {
        self.fixed_update_rate = interval;
        self
    }
    pub fn set_window_name<T>(&mut self, name: T) -> &Self 
    where T: std::fmt::Display
    {
        self.window_name = name.to_string();
        self
    }
    pub fn set_window_size<T>(&mut self, width: T, height: T) -> Result<&Self,>
    where
        T: TryInto<f64> + Display + Copy,
        <T as TryInto<f64>>::Error:Debug,
    {
        let _width = 
            match width.try_into() {
                Ok(n) => n,
                Err(e) => {
                    
                }
            }
        self
    }
    pub fn fps_max<T>(&mut self, fps: T) -> &Self 
    where 
        T: TryInto<u16> + Display + Copy,
        <T as TryInto<u16>>::Error:Debug, // ensures that the type can produce an error,
    {
        match fps.try_into() {
            Ok(n) => self.fps_settings.frame_time = interval_from_frequency(n),
            Err(e) => {
                error!("Could not set max FPS value to {}. Defaulting to: {} fps. {:?}", fps, DEFAULT_MAX_FPS, e);
                self.fps_settings.frame_time = interval_from_frequency(DEFAULT_MAX_FPS) ;
            }
        }
        self
    }
    pub fn fps_cap(&mut self, cap: Option<u16>) -> &Self
    {
        match cap {
            Some(fps) => {
                self.fps_settings.limit = true;
                self.fps_settings.frame_time = interval_from_frequency(fps)
            },
            None => {self.fps_settings.limit = false;}
        }
        self
    }
    pub fn build(self) -> EngineSettings {
        EngineSettings {
            fixed_update_rate: self.fixed_update_rate,
            fps_settings: self.fps_settings,
            window_name: self.window_name,
            window_size: self.window_size
        }
    }

}

fn interval_from_frequency<T>(frequency: T) -> Duration 
where 
    T: TryInto<f64> + Display + Copy,
    <T as TryInto<f64>>::Error:  Debug, {
    match frequency.try_into() {
        Ok(n) => {Duration::from_secs_f64(1.0/n)},
        Err(e) => {
            error!("Could not get interval from frequency value {}/s. Defaulting to: {}. {:?}", frequency, DEFAULT_MAX_INTERVAL, e);
            interval_from_frequency(DEFAULT_MAX_FPS)
        }
         
    }
    
}