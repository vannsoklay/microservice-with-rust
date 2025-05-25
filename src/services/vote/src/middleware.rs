use actix_web::{
    body::{BoxBody, EitherBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::StatusCode,
    Error, HttpMessage, HttpResponse,
};
use constant_time_eq::constant_time_eq;
use futures::future::{ok, LocalBoxFuture, Ready};
use log::{debug, warn};
use serde::Serialize;
use std::{
    env,
    rc::Rc,
    task::{Context, Poll},
};

#[derive(Clone)]
pub struct AuthMiddleware {
    public_paths: Rc<Vec<String>>,
}

impl AuthMiddleware {
    pub fn new(public_paths: Vec<String>) -> Self {
        Self {
            public_paths: Rc::new(public_paths),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    error: String,
    message: String,
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
            public_paths: Rc::clone(&self.public_paths),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    public_paths: Rc<Vec<String>>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let public_paths = Rc::clone(&self.public_paths);
        let path = req.path().to_string();
        let method = req.method().clone();
        let headers = req.headers().clone();

        let expected_secret = env::var("INTERNAL_SECRET_KEY").unwrap_or_else(|_| "".to_string());

        Box::pin(async move {
            if public_paths.iter().any(|p| path.starts_with(p)) {
                debug!("Bypassing authentication for public path: {}", path);
                let res = service.call(req).await?;
                Ok(res.map_into_left_body())
            } else {
                let user_id = headers.get("X-User-ID").and_then(|v| v.to_str().ok());
                let role = headers.get("X-User-Role").and_then(|v| v.to_str().ok());
                let service_key = headers.get("X-Service-Key").and_then(|v| v.to_str().ok());

                let is_secret_valid = match service_key {
                    Some(key) if !expected_secret.is_empty() => {
                        constant_time_eq(key.as_bytes(), expected_secret.as_bytes())
                    }
                    _ => false,
                };

                if !is_secret_valid {
                    warn!(
                        "Unauthorized access: invalid service key for {} {}",
                        method, path
                    );
                    return respond_json(
                        StatusCode::UNAUTHORIZED,
                        "unauthorized",
                        "Invalid or missing service key",
                        req,
                    );
                }

                match role {
                    Some("user") | Some("admin") => {
                        if let Some(uid) = user_id {
                            debug!("Authenticated user access: ID={}, path={}", uid, path);
                            req.extensions_mut().insert(uid.to_string());
                            let res = service.call(req).await?;
                            Ok(res.map_into_left_body())
                        } else {
                            warn!("User/admin role specified but missing X-User-ID header");
                            respond_json(
                                StatusCode::UNAUTHORIZED,
                                "unauthorized",
                                "User ID is required for role",
                                req,
                            )
                        }
                    }
                    Some(r) => {
                        warn!("Access denied: Unrecognized role '{}' for path {}", r, path);
                        respond_json(
                            StatusCode::FORBIDDEN,
                            "forbidden",
                            &format!("Access denied: unrecognized role '{}'", r),
                            req,
                        )
                    }
                    None => {
                        warn!("Access denied: Missing role header for path {}", path);
                        respond_json(
                            StatusCode::UNAUTHORIZED,
                            "unauthorized",
                            "Missing user role header",
                            req,
                        )
                    }
                }
            }
        })
    }
}

fn respond_json<B>(
    status: StatusCode,
    error: &str,
    message: &str,
    req: ServiceRequest,
) -> Result<ServiceResponse<EitherBody<B, BoxBody>>, Error> {
    let body = ErrorResponse {
        code: status.as_u16(),
        error: error.to_string(),
        message: message.to_string(),
    };

    let res = HttpResponse::build(status).json(body);
    Ok(req.into_response(res.map_into_right_body()))
}
