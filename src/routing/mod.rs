use reqwest::Client;
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod gateway;
pub mod load_balancer;

// Define backends for different services
const AUTH_BACKENDS: [&str; 1] = ["http://localhost:8089"];
const USER_BACKENDS: [&str; 1] = ["http://localhost:8083"];
const POST_BACKENDS: [&str; 1] = ["http://localhost:8088"];
const COMMENT_BACKENDS: [&str; 1] = ["http://localhost:8099"];
const VOTE_BACKENDS: [&str; 1] = ["http://localhost:8091"];
const ACCOMMODATION_BACKENDS: [&str; 1] = ["http://localhost:8081"];
const ORDER_BACKENDS: [&str; 2] = ["http://localhost:8085", "http://localhost:8086"];

// Atomic counter for round-robin load balancing per service
pub struct ServiceState {
    pub http_client: Client,
    pub auth_counter: AtomicUsize,
    pub user_counter: AtomicUsize,
    pub comment_counter: AtomicUsize,
    pub vote_counter: AtomicUsize,
    pub post_counter: AtomicUsize,
    pub accommodation_counter: AtomicUsize,
    pub order_counter: AtomicUsize,
}

impl ServiceState {
    pub fn new() -> Self {
        ServiceState {
            http_client: Client::new(),
            auth_counter: AtomicUsize::new(0),
            user_counter: AtomicUsize::new(0),
            comment_counter: AtomicUsize::new(0),
            vote_counter: AtomicUsize::new(0),
            post_counter: AtomicUsize::new(0),
            accommodation_counter: AtomicUsize::new(0),
            order_counter: AtomicUsize::new(0),
        }
    }

    // Select the next backend for each service type using round-robin
    pub fn get_next_backend(&self, service: &str) -> Option<&'static str> {
        match service {
            "auth" => {
                let index = self.auth_counter.fetch_add(1, Ordering::SeqCst) % AUTH_BACKENDS.len();
                Some(AUTH_BACKENDS[index])
            }
            "user" => {
                let index = self.user_counter.fetch_add(1, Ordering::SeqCst) % USER_BACKENDS.len();
                Some(USER_BACKENDS[index])
            }
            "post" => {
                let index = self.post_counter.fetch_add(1, Ordering::SeqCst) % POST_BACKENDS.len();
                Some(POST_BACKENDS[index])
            }
            "comment" => {
                let index =
                    self.comment_counter.fetch_add(1, Ordering::SeqCst) % COMMENT_BACKENDS.len();
                Some(COMMENT_BACKENDS[index])
            }
            "vote" => {
                let index = self.vote_counter.fetch_add(1, Ordering::SeqCst) % VOTE_BACKENDS.len();
                Some(VOTE_BACKENDS[index])
            }
            "accommodation" => {
                let index = self.accommodation_counter.fetch_add(1, Ordering::SeqCst)
                    % ACCOMMODATION_BACKENDS.len();
                Some(ACCOMMODATION_BACKENDS[index])
            }
            "order" => {
                let index =
                    self.order_counter.fetch_add(1, Ordering::SeqCst) % ORDER_BACKENDS.len();
                Some(ORDER_BACKENDS[index])
            }
            _ => None,
        }
    }
}
