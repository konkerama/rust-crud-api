use std::fs::File;
use std::io::Error;
use std::io::Read;

pub struct Config {
    pub filepath: String,
}

impl Config {
    pub fn init() -> Self {
        match std::env::var("CONFIG_DIRECTORY") {
            Ok(path) => Self { filepath: path },
            Err(_) => {
                tracing::error!("CONFIG_DIRECTORY environment variable not set");
                Self {
                    filepath: "".to_string(),
                }
            }
        }
    }

    pub fn get_config_from_env_var(&self, name: &str) -> String {
        tracing::info!("Retrieving value from environment variable {}", name);
        let env_var: String = match std::env::var(&name) {
            Ok(env_var) => env_var,
            Err(err) => {
                tracing::error!("Environment variable {} not found", name);
                tracing::error!("Error {}", err);
                std::process::exit(1);
            }
        };
        return env_var;
    }

    fn get_config_from_file(&self, name: &str) -> Result<String, Error> {
        let filepath = format!("{}{}", self.filepath, name);
        tracing::info!("Reading variable from: {}", filepath);

        let mut file = match File::open(filepath) {
            Ok(file) => file,
            Err(err) => {
                tracing::warn!("Error opening file: {:?}", err);
                return Err(err);
            }
        };

        let mut content = String::new();
        if let Err(err) = file.read_to_string(&mut content) {
            tracing::warn!("Error reading file: {:?}", err);
            return Err(err);
        }

        Ok(content)
    }

    pub fn get_config(&self, name: &str) -> String {
        if self.filepath.is_empty() {
            return self.get_config_from_env_var(name);
        }

        let value = match self.get_config_from_file(name) {
            Ok(value) => value,
            Err(_) => {
                tracing::info!(
                    "Unable to read {} from file, trying environment variable",
                    name
                );
                self.get_config_from_env_var(name)
            }
        };
        return value;
    }
}
