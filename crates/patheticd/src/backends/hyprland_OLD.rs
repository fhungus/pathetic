// This was supposed to be the code which got information fromo hyprland,
// because I wanted to pretend to be smart and do things myself!!!
// until I realized that I had LESS THAN NO IDEA what I was doing and I don't even
// really know what a socket is. I spent like 2 days on this... mann.....

use std::collections::HashMap;
use std::{
    os::unix::net::{
        UnixStream
    },
    env::var
};

use crate::backends::backend_traits::{Backend, BackendData};

fn parse_events(data: Vec<u8>) -> HashMap<String, String> {
    let mut output = HashMap::new();

    let mut parsing_key = true;
    let mut arrow_before = false;
    let mut current_key = String::new();
    let mut current_value = String::new();
    for character in data.into_iter() {
        match character {
            b'>' => {
                if arrow_before {
                    arrow_before = false;
                    parsing_key = false;
                } else {
                    arrow_before = true;
                }
            },
            b' ' => { }, // ignore whitespace
            b'\n' => {
                parsing_key = true;
                output.insert(current_key.clone(), current_value.clone());
                current_key = String::new();
                current_value = String::new();
            },
            _ => { // write to current string
                let write_to = if parsing_key { &mut current_key } else { &mut current_value };
                write_to.push(character as char);
            }
        }
    }

    return output;
}


pub struct Hyprland {
    stream: UnixStream,
    data: BackendData
}

impl Backend for Hyprland {
    fn alive(&self) -> bool {
        // if we weren't able to start then we panic
        // so this is superfluous right now
        return true;
    }

    fn init() -> Self {
        let xdg_runtime_dir: String = match var("XDG_RUNTIME_DIR") {
            Ok(dir) => dir,
            Err(error) => {
                // just panic for now
                println!("{error}");
                panic!("Failed to start backend because XDG_RUNTIME_DIR is unset.");
            }
        };

        let signature: String = match var("HYPRLAND_INSTANCE_SIGNATURE") {
            Ok(signature) => signature,
            Err(error) => {
                // just panic for now
                println!("{error}");
                panic!("Failed to start backend because HYPRLAND_INSTANCE_SIGNATURE is unset.");
            }
        };

        let stream = match UnixStream::connect(format!("{}/hypr/{}/.socket2.sock", xdg_runtime_dir, signature)) {
            Ok(stream) => stream,
            Err(error) => {
                // just panic for now
                println!("ERROR! {error}");
                panic!("Failed to start backend because the socket failed to connect");
            }
        };

        let backend = Hyprland {
            stream: stream,
        };
        
        return backend
    }

    fn poll_next(&mut self) -> BackendData {
        
    }

    // fn stall_for_update(&mut self) -> BackendData {
    //     let mut output: BackendData = HashMap::new();

    //     let mut data: Vec<u8> = Vec::new();
    //     self.stream.read_to_end(&mut data).expect("Unable to read socket data.");

    //     let mut parsing_key = true;
    //     let mut arrow_before = false;
    //     let mut current_key = String::new();
    //     let mut current_value = String::new();
    //     for character in data.into_iter() {
    //         match character {
    //             b'>' => {
    //                 if arrow_before {
    //                     arrow_before = false;
    //                     parsing_key = false;
    //                 } else {
    //                     arrow_before = true;
    //                 }
    //             },
    //             b' ' => { }, // ignore whitespace
    //             b'\n' => {
    //                 parsing_key = true;
    //                 output.insert(current_key.clone(), current_value.clone());
    //                 current_key = String::new();
    //                 current_value = String::new();
    //             },
    //             _ => { // write to current string
    //                 let write_to = if parsing_key { &mut current_key } else { &mut current_value };
    //                 write_to.push(character as char);
    //             }
    //         }
    //     }

    //     if current_key != String::new() && current_value == String::new() {
    //         println!("Warning: Socket EOF-d but still missing value");
    //     }

    //     return output;
    // }
}