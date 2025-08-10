use actix_web::HttpRequest;

// Identify function to check user id and role
pub async fn identify(req: HttpRequest) -> Option<String> {
    let user_id_header = req.headers().get("X-User-ID");
    let role_header = req.headers().get("X-User-Role");

    if let (Some(user_id), Some(role)) = (user_id_header, role_header) {
        if let Ok(role_str) = role.to_str() {
            if role_str == "user" {
                if let Ok(user_id_str) = user_id.to_str() {
                    return Some(user_id_str.to_string());
                }
            }
        }
    }

    None
}
