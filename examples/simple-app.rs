use std::path::Path;

use serde::{Deserialize, Serialize};

use ronfig::Config;

/// Your struct should at least derive `serde`'s `Serialize` and `Deserialize` to be able to read and write the ron file.
#[derive(Debug, Deserialize, Serialize)]
struct SimpleConfig {
    app_name: String,
    workers: usize,
    debug: Option<bool>,
}

impl Default for SimpleConfig {
    fn default() -> Self {
        SimpleConfig {
            app_name: "test".to_string(),
            workers: 2,
            debug: Some(false),
        }
    }
}

fn main() {
    let path = Path::new("./resources/examples/simple-config.ron");
    let result = SimpleConfig::load(path);
    let config = match result {
        Ok(simple_config) => {
            println!("Config loaded from file:\n\t {:?}", &path);
            simple_config
        }
        Err(config_error) => {
            // By implementing `Default` you can recover from `ConfigError` and
            // provide the default values to app
            println!("Error loading the config:\n\t{:?}", &config_error);
            SimpleConfig::default()
        }
    };
    println!("{:?}", &config);
}
