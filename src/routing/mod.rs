use std::sync::atomic::{AtomicUsize, Ordering};

pub mod load_balancer;

// Define backends for different services
const PRODUCT_BACKENDS: [&str; 2] = ["http://localhost:8081", "http://localhost:8082"];
const USER_BACKENDS: [&str; 2] = ["http://localhost:8083", "http://localhost:8084"];
const ORDER_BACKENDS: [&str; 2] = ["http://localhost:8085", "http://localhost:8086"];

// Atomic counter for round-robin load balancing per service
pub struct ServiceState {
    pub product_counter: AtomicUsize,
    pub user_counter: AtomicUsize,
    pub order_counter: AtomicUsize,
}

impl ServiceState {
    pub fn new() -> Self {
        ServiceState {
            product_counter: AtomicUsize::new(0),
            user_counter: AtomicUsize::new(0),
            order_counter: AtomicUsize::new(0),
        }
    }

    // Select the next backend for each service type using round-robin
    pub fn get_next_backend(&self, service: &str) -> Option<&'static str> {
        match service {
            "product" => {
                let index = self.product_counter.fetch_add(1, Ordering::SeqCst) % PRODUCT_BACKENDS.len();
                Some(PRODUCT_BACKENDS[index])
            }
            "user" => {
                let index = self.user_counter.fetch_add(1, Ordering::SeqCst) % USER_BACKENDS.len();
                Some(USER_BACKENDS[index])
            }
            "order" => {
                let index = self.order_counter.fetch_add(1, Ordering::SeqCst) % ORDER_BACKENDS.len();
                Some(ORDER_BACKENDS[index])
            }
            _ => None, // Return None if the service is not found
        }
    }
}