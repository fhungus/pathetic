use std::collections::HashMap;

use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use crate::config::Config;
use crate::error::PatheticError;
use crate::backends::backend_traits::{Backend, BackendOutput, PatheticClient};

use hyprland::event_listener::EventListener;
use hyprland::data::{Clients, Client};
use hyprland::shared::{HyprData, HyprDataActiveOptional};

pub struct Hyprland {
    data: Arc<
        Mutex<
            BackendOutput>>,
    send: mpsc::Sender<
        Arc<
            Mutex<
                BackendOutput>>>
}

fn in_do_not_show(class: &String, title: &String, do_not_show: &Vec<String>) -> bool {
    for i in do_not_show.iter() {
        if *i == *class || *i == *title { return true }
    }

    return false
}



impl Backend for Hyprland {
    fn init(config: Arc<Config>) -> Result<(
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
                focused: None
            })),
            send: send
        }));

        // get inital state of clients
        let clients = Clients::get()?;
        let backend_backend = backend.clone();

        let locked = backend_backend.lock().unwrap();
        let mut datalock = locked.data.lock().unwrap();
        for i in clients {
            // check if title or class is in configs do_not_show
            let mut show: bool = true;
            for j in config.do_not_show.iter() {
                if &i.title == j || &i.class == j {
                    show = false;
                    break
                }
            }

            if !show { continue };

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
                let config = config.clone();

                fn get_lock(backend: &Arc<Mutex<Hyprland>>) -> std::sync::MutexGuard<'_, Hyprland> {
                    return backend.lock().unwrap();
                }

                let mut event_listener = EventListener::new();

                // on focused window changed!
                let active_window_backend_backend = thread_backend.clone();
                let active_window_config = config.clone(); // i am so sorry
                event_listener.add_active_window_changed_handler(move |data| {
                    if data.is_none() {
                        println!("[HYPRLAND_BACKEND]: Active window change handler got an empty event!");
                        return;
                    }

                    let data = data.unwrap();
                    let new_window = data.address.to_string();

                    let locked = get_lock(&active_window_backend_backend);
                    let mut backend_data = locked.data.lock().unwrap();

                    if backend_data.clients.get(&data.address.to_string()).is_none() {
                        // window has not been initialized yet.
                        return;
                    }
                        
                    if !in_do_not_show(&data.class, &data.title, &active_window_config.do_not_show) {
                        backend_data.focused = Some(new_window);
                    } else {
                        backend_data.focused = None;
                    }

                    let _ = locked.send.send(locked.data.clone());
                });

                // on window open!
                let window_open_backend_backend = thread_backend.clone();
                let window_open_config = config.clone(); // i am so sorry
                event_listener.add_window_opened_handler(move |data| {
                    if in_do_not_show(&data.window_class, &data.window_title, &window_open_config.do_not_show) { return };

                    let locked = get_lock(&window_open_backend_backend);
                    let mut backend_data = locked.data.lock().unwrap();
                    backend_data.clients.insert(data.window_address.to_string(), PatheticClient {
                        title: data.window_title
                    });

                    // assuming that its focused so that we don't have to make a roundtrip
                    backend_data.focused = Some(data.window_address.to_string());

                    let _ = locked.send.send(locked.data.clone());
                });

                // on window closed!
                let window_close_backend_backend = thread_backend.clone();
                let window_close_config = config.clone(); // i am so sorry
                event_listener.add_window_closed_handler(move |address| {
                    let locked = get_lock(&window_close_backend_backend);
                    let mut backend_data = locked.data.lock().unwrap();
                    
                    if address.to_string() == backend_data.focused.clone().unwrap_or(String::new()) {
                        // get the new focused client, and check for do_not_show
                        backend_data.focused = None; // organizing it like this due to https://github.com/rust-lang/rust/issues/53667
                        let new_focused = Client::get_active().unwrap();
                        if let Some(focused) = new_focused {
                            if !in_do_not_show(&focused.class, &focused.class, &window_close_config.do_not_show) && backend_data.clients.get(&focused.address.to_string()).is_some() {
                                backend_data.focused = Some(focused.address.to_string());
                            }
                        }
                    }

                    if let Some(_) = backend_data.clients.get(&address.to_string()) {
                        backend_data.clients.remove(&address.to_string());
                    }

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