use std::{fs};
use toml::{Table, Value};

pub struct Config {
    pub address: String, // TODO: proper addresses!!
    pub key: String,
    pub backend: String,
    pub do_not_show: Vec<String>
}

// ideally should not be a function.
pub fn defaults() -> Config {
    return Config {
        address: "localhost".to_string(),
        key: "".to_string(),
        backend: "hyprland".to_string(),
        do_not_show: Vec::new()
    }
}

fn get_string(value_name: &str, value: Option<&Value>) -> Option<String> {
    match value {
        Some(Value::String(data)) => {
            return Some(data.clone());
        },
        None => {
            println!("No {} specified, using defaults.", value_name);
        },
        _ => {
            println!("{} was specified as something other than a string...", value_name);
        }
    } 

    return None
}

pub fn read_file(path: &str) -> Config {
    let mut base = defaults();

    // TODO: Implement proper error handling on this stuff
    let input = fs::read_to_string(path).expect("Config proided could not be read.");
    let parsed = input.parse::<Table>().expect("Config provided could not be parsed");

    let address_option = parsed.get("address");
    base.address = get_string("address", address_option).unwrap_or(base.address);

    let key_option = parsed.get("key");
    base.key = get_string("key", key_option).unwrap_or(base.key);

    let backend_option = parsed.get("backend");
    base.key = get_string("address", backend_option).unwrap_or(base.key);


    let do_not_show_option = parsed.get("do_not_show");
    match do_not_show_option {
        Some(Value::Array(values)) => {
            let mut new_array = Vec::new();
            for i in values.iter() {
                let stringified = i.as_str();
                if !stringified.is_some() {
                    // TODO: Proper error handling
                    println!("Found value {} in do_not_show that is not a string. Skipping.", i);
                    continue;
                }

                // iffy
                new_array.push(stringified.unwrap().to_string());
            };
            base.do_not_show = new_array;
        },
        None => { },
        _ => {
            // TODO: Proper error handling
            panic!("do_not_show is not a vector!");
        }
    };

    return base
}
