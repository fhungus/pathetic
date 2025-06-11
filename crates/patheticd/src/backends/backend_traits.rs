use crate::error::PatheticError;
use std::{collections::HashMap, sync::{Arc, Mutex, mpsc}};

pub struct PatheticClient {
    pub title: String, 
}

#[derive(Debug, Clone)]
pub struct BackendOutput {
    pub clients: HashMap<String, PatheticClient>,
    pub focused: String
}

pub trait Backend: Send + Sync
{
    fn init() -> Result<(Arc<Mutex<Self>>, mpsc::Receiver<BackendOutput>), PatheticError>;
}