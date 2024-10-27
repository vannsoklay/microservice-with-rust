use actix_web::{Error, HttpRequest, HttpResponse, Result};
use actix_service::Service;
use std::time::{Duration, Instant};
use std::sync::Mutex;
use std::collections::HashMap;

pub struct RateLimit {
    requests: Mutex<HashMap<String, (usize, Instant)>>, // Client IP -> (request count, last request time)
    limit: usize,  // Max requests allowed
    window: Duration, // Time window for rate limiting
}

impl RateLimit {
    pub fn new(limit: usize, window: Duration) -> Self {
        Self {
            requests: Mutex::new(HashMap::new()),
            limit,
            window,
        }
    }
    pub async fn check_rate_limit(&self, req: &HttpRequest) -> Result<HttpResponse, Error> {
        let ip = req.peer_addr().map(|addr| addr.ip().to_string()).unwrap_or_default();
        let mut requests = self.requests.lock().unwrap();

        let entry = requests.entry(ip.clone()).or_insert((0, Instant::now()));
        let (count, last_time) = *entry;

        if last_time.elapsed() > self.window {
            *entry = (1, Instant::now());
            Ok(HttpResponse::TooManyRequests().finish())
        } else if count < self.limit {
            *entry = (count + 1, last_time);
            Ok(HttpResponse::Ok().finish())
        } else {
            Ok(HttpResponse::TooManyRequests().finish())
        }
    }

}
