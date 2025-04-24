use std::env;

pub fn global_variables(key: String) -> String {
    let variable = env::var(key.clone())
        .expect(&format!("{} must be set in the environment", key.clone()).to_string());
    variable
}

pub fn public_service(path: &str) -> bool {
    println!("path {}", path);
    if path == "/api/v1/auth/login" || path == "/api/v1/auth/register" {
        return true;
    }
    false
}

pub fn detect_service(path: &str) -> Option<&'static str> {
    let path = path.strip_prefix("/api/v1")?;
    if path.starts_with("/auth") {
        Some("auth")
    } else if path.starts_with("/user") {
        Some("user")
    } else if path.starts_with("/accommodation") {
        Some("accommodation")
    } else if path.starts_with("/order") {
        Some("order")
    } else {
        None
    }
}

pub fn build_uri(base: &str, path: &str, query: &str) -> String {
    if query.is_empty() {
        format!("{}{}", base, path)
    } else {
        format!("{}{}?{}", base, path, query)
    }
}
