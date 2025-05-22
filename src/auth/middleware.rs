use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use crate::auth::jwt::validate_token;
use futures_util::future::{ok, Ready, LocalBoxFuture};

use std::rc::Rc;
use std::task::{Context, Poll};

pub struct AuthMiddleware;

impl<S> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<ServiceResponse, Error>>;

    fn poll_ready(&self, _ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_header = req.headers().get("Authorization").cloned();
        let service = Rc::clone(&self.service);
        let path = req.path().to_string(); // 👈 we'll use this to match role access

        Box::pin(async move {
            if let Some(header_value) = auth_header {
                if let Ok(auth_str) = header_value.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        match validate_token(token) {
                            Ok(claims) => {
                                // 🔐 Example route-based access control
                                match (path.as_str(), claims.role.as_str()) {
                                    ("/admin-area", "superadmin") => service.call(req).await,
                                    ("/admin-area", "admin") => service.call(req).await,
                                    ("/manager-area", "manager") => service.call(req).await,
                                    ("/user-area", "enduser") => service.call(req).await,
                                    ("/protected", _) => service.call(req).await,
                                    _ => Err(actix_web::error::ErrorForbidden("Insufficient role")),
                                }
                            }
                            Err(e) => {
                                println!("[AuthMiddleware] Invalid token: {:?}", e);
                                Err(actix_web::error::ErrorUnauthorized("Invalid token"))
                            }
                        }
                    } else {
                        Err(actix_web::error::ErrorUnauthorized("Missing Bearer prefix"))
                    }
                } else {
                    Err(actix_web::error::ErrorUnauthorized("Invalid Authorization header"))
                }
            } else {
                Err(actix_web::error::ErrorUnauthorized("Missing Authorization header"))
            }
        })
    }
}
