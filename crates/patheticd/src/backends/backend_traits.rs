use crate::error::PatheticError;
use std::sync::{Arc, Condvar, Mutex};

pub trait Backend
{
    fn get_updated(&self) -> &Condvar;
    fn init() -> Result<(Arc<Mutex<Self>>, Arc<Condvar>), PatheticError>;
}