
pub fn public_service(path: &str) -> bool {
    if path == "/api/v1/auth/login" || path == "/api/v1/auth/register" {
        return true;
    }
    false
}

pub fn detect_service(path: &str) -> Option<&'static str> {
    let path = path.strip_prefix("/api/v1")?;

    let services = [
        ("/auth", "auth"),
        ("/user", "user"),
        ("/follow", "follow"),
        ("/posts", "post"),
        ("/comments", "comment"),
        ("/votes", "vote"),
        ("/properties", "property"),
        ("/orders", "order"),
    ];

    for (prefix, name) in services.iter() {
        if path.starts_with(prefix) {
            return Some(*name);
        }
    }

    None
}

pub fn build_uri(base: &str, path: &str, query: &str) -> String {
    if query.is_empty() {
        format!("{}{}", base, path)
    } else {
        format!("{}{}?{}", base, path, query)
    }
}
