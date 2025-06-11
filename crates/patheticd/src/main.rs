use std::fs;

use patheticd::backends::select::get_backend;
use patheticd::config;
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

    let backend = get_backend(&config.backend);
    assert!(backend.is_ok(), "Could not start backend {}, got error.", &config.backend);
    let (backend, updated) = backend.unwrap();

    loop {
        match updated.recv() {
            Ok(data) => {
                
            },
            Err(e) => {
                warn!("{}", PatheticError::ThreadConnectionFaliure(e));
            }
        }
    }
}