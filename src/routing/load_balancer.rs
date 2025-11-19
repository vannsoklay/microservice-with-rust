// Define backends for different services
pub const AUTH_BACKENDS: [&str; 1] = ["http://localhost:8081"];
pub const USER_BACKENDS: [&str; 1] = ["http://localhost:8080"];
pub const FOLLOW_BACKENDS: [&str; 1] = ["http://localhost:8085"];
pub const POST_BACKENDS: [&str; 1] = ["http://localhost:8082"];
pub const COMMENT_BACKENDS: [&str; 1] = ["http://localhost:8083"];
pub const VOTE_BACKENDS: [&str; 1] = ["http://localhost:8084"];
pub const PROPERTY_BACKENDS: [&str; 1] = ["http://localhost:8081"];
pub const ORDER_BACKENDS: [&str; 2] = ["http://localhost:8085", "http://localhost:8086"];
