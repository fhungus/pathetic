use std::fs;

use patheticd::backends::backend_traits::Backend;
use patheticd::backends::select::get_backend;
use patheticd::config;

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
    if backend.is_err() {
        panic!("Could not start backend {}, got error.", &config.backend);
    }
    let mut backend = backend.unwrap();
}