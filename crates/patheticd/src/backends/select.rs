use crate::backends::backend_traits::{Backend, BackendOutput};
use crate::backends::hyprland::Hyprland;
use crate::config::Config;
use crate::error::PatheticError;
use std::sync::{Arc, Mutex, mpsc};

pub fn get_backend(config: Arc<Config>) -> Result<(Arc<Mutex<impl Backend>>, mpsc::Receiver<Arc<Mutex<BackendOutput>>>), PatheticError> { 
    match config.backend.as_str() {
        "hyprland" => { return Hyprland::init(config); },
        _ => { panic!("Config provided invalid backend!") } // TODO: proper error handling
    }
}