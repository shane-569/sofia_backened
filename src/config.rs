use std::env;

pub fn get_env(key: &str) -> String {
    env::var(key).expect(&format!("Missing env var: {}", key))
}