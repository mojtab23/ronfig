//! Loads RON files into a structure for easy / statically typed usage.

// #![doc(
// html_logo_url = "https://amethyst.rs/brand/logo-standard.svg",
// html_root_url = "https://docs.amethyst.rs/stable"
// )]
#![crate_name = "ronfig"]
#![warn(
missing_debug_implementations,
missing_docs,
rust_2018_idioms,
rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

use std::{
    error::Error,
    fmt, io,
    path::{Path, PathBuf},
};

use ron::{self, error::Error as RonError};
use serde::{Deserialize, Serialize};

/// Error related to anything that manages/creates configurations as well as
/// "workspace"-related things.
#[derive(Debug)]
pub enum ConfigError {
    /// Forward to the `std::io::Error` error.
    File(io::Error),
    /// Errors related to serde's parsing of configuration files.
    Parser(ron::Error),
    /// Occurs if a value is ill-formed during serialization (like a poisoned mutex).
    Serializer(ron::Error),
    /// Related to the path of the file.
    Extension(PathBuf),
}

/// Config file format for serde
#[derive(Debug)]
pub enum ConfigFormat {
    /// Rusty Object Notation files (.ron), default
    Ron,
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ConfigError::File(ref err) => write!(f, "{}", err),
            ConfigError::Parser(ref msg) => write!(f, "{}", msg),
            ConfigError::Serializer(ref msg) => write!(f, "{}", msg),
            ConfigError::Extension(ref path) => {
                let found = match path.extension() {
                    Some(extension) => format!("{:?}", extension),
                    None => "a directory.".to_string(),
                };

                write!(
                    f,
                    "{}: Invalid path extension, expected \"ron\", got {}.",
                    path.display().to_string(),
                    found,
                )
            }
        }
    }
}

impl From<RonError> for ConfigError {
    fn from(e: RonError) -> Self {
        ConfigError::Parser(e)
    }
}


impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> ConfigError {
        ConfigError::File(e)
    }
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        match *self {
            ConfigError::File(_) => "Project file error",
            ConfigError::Parser(_) => "Project parser error",
            ConfigError::Serializer(_) => "Project serializer error",
            ConfigError::Extension(_) => "Invalid extension or directory for a file",
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match *self {
            ConfigError::File(ref err) => Some(err),
            _ => None,
        }
    }
}

/// Trait implemented by the `config!` macro.
pub trait Config where Self: Sized, {
    /// Loads a configuration structure from a file.
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError>;

    /// Loads a configuration structure from a file.
    // #[deprecated(note = "use `load` instead")]
    // fn load_no_fallback<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
    //     Self::load(path)
    // }

    /// Loads configuration structure from raw bytes.
    fn load_bytes_format(format: ConfigFormat, bytes: &[u8]) -> Result<Self, ConfigError>;

    /// Writes a configuration structure to a file.
    fn write_format<P: AsRef<Path>>(
        &self,
        format: ConfigFormat,
        path: P,
    ) -> Result<(), ConfigError>;

    /// Writes a configuration structure to a file using Ron format.
    #[deprecated(note = "use `write_format` instead")]
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        self.write_format(ConfigFormat::Ron, path)
    }
}

impl<T> Config for T
    where
        T: for<'a> Deserialize<'a> + Serialize,
{
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        use std::{fs::File, io::Read};

        use encoding_rs_io::DecodeReaderBytes;

        let path = path.as_ref();

        let content = {
            let file = File::open(path)?;

            // Convert UTF-8-BOM & UTF-16-BOM to regular UTF-8. Else bytes are passed through
            let mut decoder = DecodeReaderBytes::new(file);

            let mut buffer = Vec::new();
            decoder.read_to_end(&mut buffer)?;

            buffer
        };

        if let Some(extension) = path.extension().and_then(std::ffi::OsStr::to_str) {
            match extension {
                "ron" => Self::load_bytes_format(ConfigFormat::Ron, &content),
                _ => Err(ConfigError::Extension(path.to_path_buf())),
            }
        } else {
            Err(ConfigError::Extension(path.to_path_buf()))
        }
    }

    fn load_bytes_format(format: ConfigFormat, bytes: &[u8]) -> Result<Self, ConfigError> {
        match format {
            ConfigFormat::Ron => {
                ron::de::Deserializer::from_bytes(bytes)
                    .and_then(|mut de| {
                        let val = T::deserialize(&mut de)?;
                        de.end()?;
                        Ok(val)
                    })
                    .map_err(ConfigError::Parser)
            }
        }
    }

    fn write_format<P: AsRef<Path>>(
        &self,
        format: ConfigFormat,
        path: P,
    ) -> Result<(), ConfigError> {
        use std::{fs::File, io::Write};

        match format {
            ConfigFormat::Ron => {
                let str = ron::ser::to_string_pretty(self, Default::default())?;
                File::create(path)?.write_all(str.as_bytes())?;
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use serde::{Deserialize, Serialize};

    use crate::Config;

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct TestConfig {
        amethyst: bool,
    }

    #[test]
    fn load_file() {
        let expected = TestConfig { amethyst: true };

        let parsed =
            TestConfig::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/test/config.ron"));

        assert_eq!(expected, parsed.unwrap());
    }

    #[test]
    fn load_file_with_bom_encodings() {
        let expected = TestConfig { amethyst: true };

        let utf8_bom =
            TestConfig::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/test/UTF8-BOM.ron"));
        let utf16_le_bom =
            TestConfig::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/test/UTF16-LE-BOM.ron"));
        let utf16_be_bom =
            TestConfig::load(Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/test/UTF16-BE-BOM.ron"));

        assert_eq!(
            expected,
            utf8_bom.expect("Failed to parse UTF8 file with BOM")
        );
        assert_eq!(
            expected,
            utf16_le_bom.expect("Failed to parse UTF16-LE file with BOM")
        );
        assert_eq!(
            expected,
            utf16_be_bom.expect("Failed to parse UTF16-BE file with BOM")
        );
    }
}