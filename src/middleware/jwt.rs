use actix_service::{Service, Transform};
use actix_web::{
    dev::{ServiceRequest, ServiceResponse},
    Error, HttpMessage as _,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::{
    auth::{verify_jwt_from_header, Claims},
    utils::public_service,
};

pub struct JwtMiddleware {
    pub secret: String,
}

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
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
            secret: self.secret.clone(),
        })
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
    secret: String,
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
        let secret = self.secret.clone();

        // Don't verify for public endpoints
        let path = req.path().to_string();
        let public = public_service(&path);

        println!("public {:?}", public);

        Box::pin(async move {
            if !public {
                match verify_jwt_from_header(&req, &secret) {
                    Ok(claims) => {
                        req.extensions_mut().insert::<Claims>(claims);
                    }
                    Err(_) => {
                        // Instead of returning an error, insert "guest" claims
                        let guest_claims = Claims {
                            sub: "guest".into(),
                            exp: 0, // Optionally set an expiration if needed
                            role: "guest".into(),
                            // add any other default fields required by your Claims struct
                        };
                        req.extensions_mut().insert::<Claims>(guest_claims);
                    }
                }
            }
            println!("work public access...");
            service.call(req).await
        })
    }
}
