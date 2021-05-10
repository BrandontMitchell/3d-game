use std::collections::HashMap;
use kira::sound::handle::SoundHandle;
use kira::instance::InstanceSettings;
use kira::sound::SoundSettings;
use kira::manager::AudioManagerSettings;
use kira::manager::AudioManager;
use kira::manager::error::SetupError;


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