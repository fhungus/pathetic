use std::collections::HashMap;

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use crate::error::PatheticError;
use crate::backends::backend_traits::{Backend, BackendOutput, PatheticClient};

use hyprland::event_listener::EventListener;
use hyprland::data::Clients;
use hyprland::shared::{HyprData};

pub struct Hyprland {
    data: Arc<
        Mutex<
            BackendOutput>>,
    send: mpsc::Sender<
        Arc<
            Mutex<
                BackendOutput>>>
}

impl Backend for Hyprland {
    fn init() -> Result<(
        Arc<
            Mutex<
                Self>>, 
        mpsc::Receiver<
            Arc<
                Mutex<
                    BackendOutput>>>
    ),
    PatheticError> 
    {
        let (send, receive) = mpsc::channel();
        let backend = Arc::new(Mutex::new( Hyprland {
            data: Arc::new(Mutex::new(BackendOutput {
                clients: HashMap::new(),
                focused: "".to_string()
            })),
            send: send
        }));

        // get inital state of clients
        let clients = Clients::get()?;
        let backend_backend = backend.clone();

        let locked = backend_backend.lock().unwrap();
        // i don't know if i trust rust to destroy the lock nor do i trust it to unwrap successfully consistently
        let mut datalock = locked.data.lock().unwrap();
        for i in clients {
            let client = PatheticClient {
                title: i.title,
            };

            datalock.clients.insert(i.address.to_string(), client);
        }

        drop(datalock);
        drop(locked);

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
                let active_window_backend_backend = thread_backend.clone();
                event_listener.add_active_window_changed_handler(move |data| {
                    if data.is_none() {
                        println!("[HYPRLAND_BACKEND]: Active window change handler got an empty event!");
                        return;
                    }
                    let data = data.unwrap();
                    let new_window = data.address.to_string();

                    let locked = get_lock(&active_window_backend_backend);
                    let mut backend_data = locked.data.lock().unwrap();
                    backend_data.focused = new_window;
                    drop(data);

                    let _ = locked.send.send(locked.data.clone());
                });

                // on window closed!
                let window_close_backend_backend = thread_backend.clone();
                event_listener.add_window_closed_handler(move |address| {
                    let locked = get_lock(&window_close_backend_backend);
                    let mut backend_data = locked.data.lock().unwrap();
                     
                    if let Some(_) = backend_data.clients.get(&address.to_string()) {
                        backend_data.clients.remove(&address.to_string());
                    } else {
                        let address: String = address.to_string();
                        println!("[HYPRLAND_BACKEND]: Closed window {address} was not in Clients list.")
                    }

                    let _ = locked.send.send(locked.data.clone());
                });
                
                // on window open!
                let window_open_backend_backend = thread_backend.clone();
                event_listener.add_window_opened_handler(move |data| {
                    let locked = get_lock(&window_open_backend_backend);
                    let mut backend_data = locked.data.lock().unwrap();
                    backend_data.clients.insert(data.window_address.to_string(), PatheticClient {
                        title: data.window_title
                    });

                    // assuming that its focused so that we don't have to make a roundtrip
                    backend_data.focused = data.window_address.to_string();
                    let _ = locked.send.send(locked.data.clone());
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