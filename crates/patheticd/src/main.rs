use std::fs;

use hyprland::shared::{HyprData, Address, HyprDataActive, HyprDataActiveOptional};
use hyprland::data::Clients;
use patheticd::backends::select::get_backend;
use patheticd::config;

// epic procmacro because im a big stupid baby
#[warn(clippy::pedantic, clippy::nursery)]
fn main() {
    let paths: Vec<&str> = vec![
        "./patheticd.toml",
        "~/.config/pathetic/patheticd.toml",
        "~/.config/patheticd.toml"
    ];
    
    let clients = Clients::get().unwrap();
    
    let mut config = config::defaults();
    for i in paths {
        if fs::exists(i).unwrap() {
            config = config::read_file(i);
        }
    }

    let backend = get_backend(&config.backend);
    assert!(backend.is_ok(), "Could not start backend {}, got error.", &config.backend);

    let (mut backend, updated) = backend.unwrap();
}