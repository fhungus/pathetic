use std::collections::HashMap;

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use crate::error::PatheticError;
use crate::backends::backend_traits::{Backend, BackendOutput, PatheticClient};

use hyprland::event_listener::EventListener;
use hyprland::data::Clients;
use hyprland::shared::{HyprData};

pub struct Hyprland {
    data: Mutex<BackendOutput>,
    send: mpsc::Sender<BackendOutput>
}

impl Backend for Hyprland {
    fn init() -> Result<(Arc<Mutex<Self>>, mpsc::Receiver<BackendOutput>), PatheticError> {
        let (send, receive) = mpsc::channel();
        let backend = Arc::new(Mutex::new( Hyprland {
            data: BackendOutput {
                clients: HashMap::new(),
                focused: "".to_string()
            },
            send: send
        }));

        // get inital state of clients
        let clients = Clients::get()?;

        for i in clients {
            let client = PatheticClient {
                title: i.title,
            };

            let mut locked = backend.lock().unwrap();
            locked.data.clients.insert(i.address.to_string(), client);
        }


        // set up event thread
        let thread_backend = backend.clone();
        let result = thread::Builder::new()
            .name("event-updater".to_string())
            .spawn(move || {
                fn get_lock(backend: &Arc<Mutex<Hyprland>>) -> std::sync::MutexGuard<'_, Hyprland> {
                    return backend.lock().unwrap();
                }

                let mut event_listener = EventListener::new();

                // on focused window changed!
                let active_window_thread_backend = thread_backend.clone();
                event_listener.add_active_window_changed_handler(move |data| {
                    if data.is_none() {
                        println!("[HYPRLAND_BACKEND]: Active window change handler got an empty event!");
                        return;
                    }
                    let data = data.unwrap();
                    let new_window = data.address.to_string();

                    let mut locked = get_lock(&active_window_thread_backend);
                    locked.data.focused = new_window;
                    locked.send.send(locked.data);
                });

                // on window closed!
                let window_close_thread_backend = thread_backend.clone();
                event_listener.add_window_closed_handler(move |data| {
                    let mut locked = get_lock(&window_close_thread_backend);
                    let found = locked.data.clients.get(&data.to_string());

                    if found.is_some() {
                        locked.data.clients.remove(&data.to_string());
                    } else {
                        let address = data.to_string();
                        println!("[HYPRLAND_BACKEND]: Closed window {address} was not in Clients list.")
                    }
                    locked.send.send(locked.data);
                });
                
                // on window open!
                let window_open_thread_backend = thread_backend.clone();
                event_listener.add_window_opened_handler(move |data| {
                    let mut locked = get_lock(&window_open_thread_backend);
                    locked.data.clients.insert(data.window_address.to_string(), PatheticClient {
                        title: data.window_title
                    });

                    // assuming that its focused so that we don't have to make a roundtrip
                    locked.data.focused = data.window_address.to_string();
                    locked.send.send(locked.data);
                });

                // TODO: Get errors to main if the thread fails.
                let _ = event_listener.start_listener();
            });

        if result.is_err() {
            return Err(PatheticError::ThreadInitFailiure(result.unwrap_err()))
        }

        return Ok((backend, receive))
    }
}