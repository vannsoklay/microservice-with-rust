use std::env;

pub fn global_variables(key: String) -> String {
    let variable = env::var(key.clone())
        .expect(&format!("{} must be set in the environment", key.clone()).to_string());
    variable
}