use std::fmt::Debug;

use std::io::Error;
use std::sync::mpsc::RecvError;
use hyprland::shared::HyprError;

#[derive(Debug, derive_more::Display)]
pub enum PatheticError {
    HyprError(HyprError), // idk if i should have a type for each backend in the future or if i should have just one type for all... 
    ThreadInitFailiure(Error),
    ThreadConnectionFaliure(RecvError),
    ServerConnectionFailed
}

impl From<HyprError> for PatheticError {
    fn from(value: HyprError) -> Self {
        return PatheticError::HyprError(value);
    }
}