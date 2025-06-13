use crate::backends::backend_traits::{Backend, BackendOutput};
use crate::backends::hyprland::Hyprland;
use crate::error::PatheticError;
use std::sync::{Arc, Mutex, mpsc};

pub fn get_backend(id: &str) -> Result<(Arc<Mutex<impl Backend>>, mpsc::Receiver<Arc<Mutex<BackendOutput>>>), PatheticError> { 
    match id {
        "hyprland" => { return Hyprland::init(); },
        _ => { panic!("No backend named {id}") } // TODO: proper error handling
    }
}