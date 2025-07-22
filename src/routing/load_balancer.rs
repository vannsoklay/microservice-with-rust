// Define backends for different services
pub const AUTH_BACKENDS: [&str; 1] = ["http://localhost:8089"];
pub const USER_BACKENDS: [&str; 1] = ["http://localhost:8083"];
pub const FOLLOW_BACKENDS: [&str; 1] = ["http://localhost:9011"];
pub const POST_BACKENDS: [&str; 1] = ["http://localhost:8088"];
pub const COMMENT_BACKENDS: [&str; 1] = ["http://localhost:8099"];
pub const VOTE_BACKENDS: [&str; 1] = ["http://localhost:8091"];
pub const ACCOMMODATION_BACKENDS: [&str; 1] = ["http://localhost:8081"];
pub const ORDER_BACKENDS: [&str; 2] = ["http://localhost:8085", "http://localhost:8086"];
