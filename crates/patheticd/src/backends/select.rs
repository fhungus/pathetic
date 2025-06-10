use crate::backends::backend_traits::Backend;
use crate::backends::hyprland::Hyprland;
use crate::error::PatheticError;
use std::sync::{Arc, Mutex};

pub fn get_backend(id: &str) -> Result<Arc<Mutex<impl Backend>>, PatheticError> { 
    match id {
        "hyprland" => { return Hyprland::init(); },
        _ => { panic!("No backend named {id}") }
    }
}