use crate::{config::Config, error::PatheticError};
use std::{collections::HashMap, sync::{Arc, Mutex, mpsc}};

pub struct PatheticClient {
    pub title: String, 
}

pub struct BackendOutput {
    pub clients: HashMap<String, PatheticClient>,
    pub focused: Option<String>
}

pub trait Backend: Send + Sync
{
    fn init(config: Arc<Config>) -> Result<(
        Arc<
            Mutex<
                Self>>, 
        mpsc::Receiver<
            Arc<
                Mutex<
                    BackendOutput>>>), 
        PatheticError>;
}