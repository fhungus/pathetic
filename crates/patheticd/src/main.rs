use std::fs;
use std::sync::Arc;

use patheticd::backends::select::get_backend;
use patheticd::config::{self, Config};
use patheticd::error::PatheticError;

// epic procmacro because im a big stupid baby
#[warn(clippy::pedantic, clippy::nursery)]
fn main() {
    let paths: Vec<&str> = vec![
        "./patheticd.toml",
        "~/.config/pathetic/patheticd.toml",
        "~/.config/patheticd.toml"
    ];
    
    let mut config = config::defaults();
    for i in paths {
        if fs::exists(i).unwrap() {
            config = config::read_file(i);
        }
    }

    let shared_config: Arc<Config> = Arc::new(config);

    let backend = get_backend(shared_config.clone());
    assert!(backend.is_ok(), "Could not start backend {}, got error.", &shared_config.backend);
    let (_backend, updated) = backend.unwrap();

    loop {
        match updated.recv() {
            Ok(data) => {
                let locked = data.lock().unwrap();
                match &locked.focused {
                    Some(i) => {
                        println!("focused window: {}", locked.clients.get(i).unwrap().title);
                    },
                    None => {
                        println!("lets justr say... im leagueing it")
                    }
                }
            },
            Err(e) => {
                println!("{}", PatheticError::ThreadConnectionFaliure(e));
            }
        }
    }
}