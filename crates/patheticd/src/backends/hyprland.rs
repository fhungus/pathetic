use std::collections::HashMap;

use std::sync::{Arc, Mutex, Condvar};
use std::thread;

use crate::error::PatheticError;
use crate::backends::backend_traits::Backend;

use hyprland::event_listener::EventListener;
use hyprland::data::Clients;
use hyprland::shared::{HyprData, Address};

struct PatheticClient {
    title: String, 
}

pub struct Hyprland {
    clients: HashMap<Address, PatheticClient>,
    focus: Address,
}

impl Backend for Hyprland {
    fn get_updated(&self) -> &Condvar {
        return &self.updated;
    }

    fn init() -> Result<(Arc<Mutex<Self>>, Condvar), PatheticError> {
        let backend = Arc::new(Mutex::new(Hyprland {
            clients: HashMap::new(),
            focus: Address::new("".to_string()),
            updated: Condvar::new()
        }));

        // get inital state of clients
        let clients = Clients::get();
        if clients.is_err() { return Err(PatheticError::HyprError(clients.unwrap_err())); }
        let clients = clients.unwrap();

        for i in clients {
            let client = PatheticClient {
                title: i.title,
            };

            let mut locked = backend.lock().unwrap();
            locked.clients.insert(i.address, client);
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

                let active_window_thread_backend = thread_backend.clone();
                event_listener.add_active_window_change_handler(move |data| {
                    if data.is_none() {
                        println!("[HYPRLAND_BACKEND]: Active window change handler got an empty event!");
                        return;
                    }
                    let data = data.unwrap();
                    let new_window = data.window_address;

                    let mut locked = get_lock(&active_window_thread_backend);
                    locked.focus = new_window;
                    locked.updated.notify_all();
                });

                let window_close_thread_backend = thread_backend.clone();
                event_listener.add_window_close_handler(move |data| {
                    let mut locked = get_lock(&window_close_thread_backend);
                    let found = locked.clients.get(&data);

                    if found.is_some() {
                        locked.clients.remove(&data);
                    } else {
                        let address = data.to_string();
                        println!("[HYPRLAND_BACKEND]: Closed window {address} was not in Clients list.")
                    }

                    locked.updated.notify_all();
                });
                
                let window_open_thread_backend = thread_backend.clone();
                event_listener.add_window_open_handler(move |data| {
                    let mut locked = get_lock(&window_open_thread_backend);
                    locked.clients.insert(data.window_address.clone(), PatheticClient {
                        title: data.window_title
                    });

                    // assuming that its focused so that we don't have to make a roundtrip
                    locked.focus = data.window_address;
                    locked.updated.notify_all();
                });

                // TODO: Get errors to main if the thread fails.
                let _ = event_listener.start_listener();
            });

        if result.is_err() {
            return Err(PatheticError::ThreadInitFailiure(result.unwrap_err()))
        }

        return Ok(backend)
    }
}