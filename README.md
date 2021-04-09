# RONFIG

Config rust with RON, Easily!

For now, it's just a copy of [amethyst_config](https://github.com/amethyst/amethyst/tree/main/amethyst_config).

## Usage


Add this to `Cargo.toml` 
```toml
[dependencies]
ronfig = "0.1"
```

Example RON file
```ron
(
    app_name:"simple app",
    workers: 4,
    debug: Some(true),
)
```


```rust
use std::path::Path;
use serde::{Deserialize, Serialize};
use ronfig::Config;

/// Your struct should at least derive `serde`'s `Serialize` and `Deserialize` to
/// be able to read and write the ron file.
#[derive(Debug, Deserialize, Serialize)]
struct SimpleConfig {
    app_name: String,
    workers: usize,
    debug: Option<bool>,
}

fn main() {
    let path = Path::new("./resources/examples/simple-config.ron");
    let result = SimpleConfig::load(path);
    match result {
        Ok(simple_config) => {
            println!("Config loaded from file:\n\t {:?}", &path);
            println!("{:?}", &simple_config);
        }
        Err(config_error) => {
            println!("Error loading the config:\n\t{:?}", &config_error);
        }
    };
}
```

