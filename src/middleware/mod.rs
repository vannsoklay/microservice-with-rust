use actix_service::{Service, Transform};
use actix_web::{dev::{ServiceRequest, ServiceResponse}, Error};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};
use std::rc::Rc;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        Box::pin(async move {
            // Check for the presence of an authorization header
            if let Some(auth_header) = req.headers().get("Authorization") {
                // Validate the token (add your own logic here, e.g., JWT verification)
                if auth_header.to_str().unwrap_or("").starts_with("Bearer ") {
                    // Token is valid (for example)
                    service.call(req).await
                } else {
                    // Token is invalid
                    Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                }
            } else {
                // No token found
                Err(actix_web::error::ErrorUnauthorized("Authorization header missing"))
            }
        })
    }
}
