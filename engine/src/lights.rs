use crate::geom::*;
use std::collections::HashMap;
use kira::sound::handle::SoundHandle;
use kira::instance::InstanceSettings;
use kira::sound::SoundSettings;
use kira::manager::AudioManagerSettings;
use kira::manager::AudioManager;
use kira::manager::error::SetupError;


#[derive(Debug, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct Light {
    pub pos: [f32; 4],
    pub dir:[f32;4],
    pub color: [f32; 4],
}
impl Light {
    pub fn point(pos: Pos3, color: Vec3) -> Self {
        Self {
            pos: [pos.x, pos.y, pos.z, 1.0],
            dir:[0.0,0.0,0.0,0.0],
            color: [color.x, color.y, color.z, 0.0],
        }
    }
    pub fn directed(dir:Vec3, color:Vec3) -> Self {
        Self {
            pos:[0.0,0.0,0.0,0.0],
            dir:[dir.x,dir.y,dir.z,1.0],
            color:[color.x,color.y,color.z, 0.0],
        }
    }
    pub fn spot(pos:Pos3, dir:Vec3, color:Vec3) -> Self {
        Self {
            pos:[pos.x,pos.y,pos.z,1.0],
            dir:[dir.x,dir.y,dir.z,1.0],
            color:[color.x,color.y,color.z, 0.0],
        }
    }

    pub fn position(&self) -> Pos3 {
        Pos3::new(self.pos[0], self.pos[1], self.pos[2])
    }
    pub fn color(&self) -> Vec3 {
        Vec3::new(self.color[0], self.color[1], self.color[2])
    }
}


pub struct Sound {
    sound_map: HashMap<String, SoundHandle>,
    manager: Option<AudioManager>,
}

impl Sound {
    pub fn new() -> Self {
        let sound_map:HashMap<String, SoundHandle> = HashMap::new();
        let manager:Option<AudioManager> = None;
        Self{
            sound_map: sound_map,
            manager: manager,
        }
    }
    pub fn init_manager(&mut self) -> Result<String, SetupError> {
        let result = AudioManager::new(AudioManagerSettings::default())?;
        self.manager = Some(result);
        Ok("cool".to_string())
    }
    pub fn add_sound(&mut self, name: String, path: String) {
        let manager_o = &mut self.manager;
        match manager_o {
            Some(manager) => {
                let handler_r = manager.load_sound(path, SoundSettings::default());
                match handler_r {
                    Ok(handler) => {self.sound_map.insert(name, handler);},
                    _ => println!("load sound error"),
                }
            },
            None => println!("missing manager"),
        }
    }
    pub fn play_sound(&mut self, name: String) {
        let map_element = self.sound_map.get_mut(&name);
        match map_element {
            Some(sound_handle) => {let _ = sound_handle.play(InstanceSettings::default());},
            None => println!("missing sound"), 
        }
    }
}