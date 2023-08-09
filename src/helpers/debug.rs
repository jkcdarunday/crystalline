use std::env;

pub fn is_debug() -> bool{
    let debug_value = env::var("DEBUG").unwrap_or_default();

    debug_value == "1" || debug_value == "true"
}
