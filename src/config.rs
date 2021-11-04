use std::env;

pub struct Config;

impl Config {
    pub fn get_host() -> String {
        Config::get_any("HYPERION_HOST", "0.0.0.0")
    }

    pub fn get_port() -> String {
        Config::get_any("HYPERION_PORT", "2310")
    }

    pub fn get_any(key: &str, fallback: &str) -> String {
        match env::var(key) {
            Ok(res) => res,
            Err(err) => fallback.to_owned(),
        }
    }
}
