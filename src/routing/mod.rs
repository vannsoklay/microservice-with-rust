use reqwest::Client;
use std::sync::atomic::{AtomicUsize, Ordering};

pub mod gateway;
pub mod load_balancer;

// Atomic counter for round-robin load balancing per service
pub struct ServiceState {
    pub http_client: Client,
    pub service_key: String,
    pub auth_counter: AtomicUsize,
    pub user_counter: AtomicUsize,
    pub follow_counter: AtomicUsize,
    pub comment_counter: AtomicUsize,
    pub vote_counter: AtomicUsize,
    pub post_counter: AtomicUsize,
}

impl ServiceState {
    pub fn new(service_key: String) -> Self {
        ServiceState {
            http_client: Client::new(),
            service_key,
            auth_counter: AtomicUsize::new(0),
            user_counter: AtomicUsize::new(0),
            follow_counter: AtomicUsize::new(0),
            comment_counter: AtomicUsize::new(0),
            vote_counter: AtomicUsize::new(0),
            post_counter: AtomicUsize::new(0),
        }
    }

    // Select the next backend for each service type using round-robin
    pub fn get_next_backend(&self, service: &str) -> Option<&'static str> {
        match service {
            "auth" => {
                let index = self.auth_counter.fetch_add(1, Ordering::SeqCst)
                    % load_balancer::AUTH_BACKENDS.len();
                Some(load_balancer::AUTH_BACKENDS[index])
            }
            "user" => {
                let index = self.user_counter.fetch_add(1, Ordering::SeqCst)
                    % load_balancer::USER_BACKENDS.len();
                Some(load_balancer::USER_BACKENDS[index])
            }
            "follow" => {
                let index = self.follow_counter.fetch_add(1, Ordering::SeqCst)
                    % load_balancer::FOLLOW_BACKENDS.len();
                Some(load_balancer::FOLLOW_BACKENDS[index])
            }
            "post" => {
                let index = self.post_counter.fetch_add(1, Ordering::SeqCst)
                    % load_balancer::POST_BACKENDS.len();
                Some(load_balancer::POST_BACKENDS[index])
            }
            "comment" => {
                let index = self.comment_counter.fetch_add(1, Ordering::SeqCst)
                    % load_balancer::COMMENT_BACKENDS.len();
                Some(load_balancer::COMMENT_BACKENDS[index])
            }
            "vote" => {
                let index = self.vote_counter.fetch_add(1, Ordering::SeqCst)
                    % load_balancer::VOTE_BACKENDS.len();
                Some(load_balancer::VOTE_BACKENDS[index])
            }
            _ => None,
        }
    }
}
