use std::{
    future::{ready, Ready},
    rc::Rc,
};

use actix_web::{
    dev::{self, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};

use futures_util::future::LocalBoxFuture;
use crate::api::users::helper::auth_token::{self, Claims, AuthConfig};

// New JWT-based authentication middleware
pub struct JwtAuth {
    auth_config: Rc<AuthConfig>,
}

impl JwtAuth {
    pub fn new() -> Self {
        Self {
            auth_config: Rc::new(AuthConfig::default()),
        }
    }

    pub fn with_config(config: AuthConfig) -> Self {
        Self {
            auth_config: Rc::new(config),
        }
    }
}

impl<S: 'static, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JwtAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Rc::new(service),
            auth_config: self.auth_config.clone(),
        }))
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
    auth_config: Rc<AuthConfig>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_config = self.auth_config.clone();
        let srv = self.service.clone();

        Box::pin(async move {
            // Extract JWT token from Authorization header
            let headers = req.headers();
            let auth_header = match headers.get("Authorization") {
                Some(auth_header) => auth_header,
                None => return Err(ErrorUnauthorized("Missing Authorization header")),
            };

            let auth_header = match auth_header.to_str() {
                Ok(auth_header) => auth_header,
                Err(_) => return Err(ErrorUnauthorized("Invalid Authorization header format")),
            };

            // Check if it's a Bearer token
            if !auth_header.starts_with("Bearer ") {
                return Err(ErrorUnauthorized("Invalid Authorization format"));
            }

            // Extract the token without "Bearer " prefix
            let token = &auth_header[7..];

            // Validate JWT token
            let claims = match auth_token::validate_token(token, &auth_config) {
                Ok(claims) => claims,
                Err(_) => return Err(ErrorUnauthorized("Invalid or expired token")),
            };

            // Store claims in request extensions for access in handlers
            req.extensions_mut().insert(claims);

            // Call the next service
            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

// Role-based authorization middleware
pub struct RoleAuth {
    auth_config: Rc<AuthConfig>,
    required_role: String,
}

impl RoleAuth {
    pub fn admin() -> Self {
        Self {
            auth_config: Rc::new(AuthConfig::default()),
            required_role: "admin".to_string(),
        }
    }

    pub fn staff() -> Self {
        Self {
            auth_config: Rc::new(AuthConfig::default()),
            required_role: "staff".to_string(),
        }
    }

    pub fn user() -> Self {
        Self {
            auth_config: Rc::new(AuthConfig::default()),
            required_role: "user".to_string(),
        }
    }

    pub fn with_role(role: &str, config: AuthConfig) -> Self {
        Self {
            auth_config: Rc::new(config),
            required_role: role.to_string(),
        }
    }
}

impl<S: 'static, B> Transform<S, ServiceRequest> for RoleAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RoleAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RoleAuthMiddleware {
            service: Rc::new(service),
            auth_config: self.auth_config.clone(),
            required_role: self.required_role.clone(),
        }))
    }
}

pub struct RoleAuthMiddleware<S> {
    service: Rc<S>,
    auth_config: Rc<AuthConfig>,
    required_role: String,
}

impl<S, B> Service<ServiceRequest> for RoleAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_config = self.auth_config.clone();
        let required_role = self.required_role.clone();
        let srv = self.service.clone();

        Box::pin(async move {
            // Extract JWT token from Authorization header
            let headers = req.headers();
            let auth_header = match headers.get("Authorization") {
                Some(auth_header) => auth_header,
                None => return Err(ErrorUnauthorized("Missing Authorization header")),
            };

            let auth_header = match auth_header.to_str() {
                Ok(auth_header) => auth_header,
                Err(_) => return Err(ErrorUnauthorized("Invalid Authorization header format")),
            };

            // Check if it's a Bearer token
            if !auth_header.starts_with("Bearer ") {
                return Err(ErrorUnauthorized("Invalid Authorization format"));
            }

            // Extract the token without "Bearer " prefix
            let token = &auth_header[7..];

            // Validate JWT token
            let claims = match auth_token::validate_token(token, &auth_config) {
                Ok(claims) => claims,
                Err(_) => return Err(ErrorUnauthorized("Invalid or expired token")),
            };

            // Check if user has required role
            let has_required_role = match required_role.as_str() {
                "admin" => auth_token::is_admin(&claims.role),
                "staff" => auth_token::is_staff_or_admin(&claims.role),
                "user" => auth_token::is_user_or_above(&claims.role),
                _ => false,
            };

            if !has_required_role {
                return Err(ErrorUnauthorized("Insufficient privileges"));
            }

            // Store claims in request extensions for access in handlers
            req.extensions_mut().insert(claims);

            // Call the next service
            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

// User self-check middleware
pub struct UserSelfCheck {
    auth_config: Rc<AuthConfig>,
    allow_admin: bool,
    allow_staff: bool,
}

impl UserSelfCheck {
    pub fn strict_self() -> Self {
        Self {
            auth_config: Rc::new(AuthConfig::default()),
            allow_admin: false,
            allow_staff: false,
        }
    }

    pub fn self_and_staff() -> Self {
        Self {
            auth_config: Rc::new(AuthConfig::default()),
            allow_admin: true, 
            allow_staff: true,
        }
    }

    pub fn self_and_admin() -> Self {
        Self {
            auth_config: Rc::new(AuthConfig::default()),
            allow_admin: true,
            allow_staff: false,
        }
    }

    pub fn with_config(config: AuthConfig, allow_admin: bool, allow_staff: bool) -> Self {
        Self {
            auth_config: Rc::new(config),
            allow_admin,
            allow_staff,
        }
    }
}

impl<S: 'static, B> Transform<S, ServiceRequest> for UserSelfCheck
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = UserSelfCheckMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UserSelfCheckMiddleware {
            service: Rc::new(service),
            auth_config: self.auth_config.clone(),
            allow_admin: self.allow_admin,
            allow_staff: self.allow_staff,
        }))
    }
}

pub struct UserSelfCheckMiddleware<S> {
    service: Rc<S>,
    auth_config: Rc<AuthConfig>,
    allow_admin: bool,
    allow_staff: bool,
}

impl<S, B> Service<ServiceRequest> for UserSelfCheckMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let auth_config = self.auth_config.clone();
        let allow_admin = self.allow_admin;
        let allow_staff = self.allow_staff;
        let srv = self.service.clone();

        Box::pin(async move {
            // Extract JWT token from Authorization header
            let headers = req.headers();
            let auth_header = match headers.get("Authorization") {
                Some(auth_header) => auth_header,
                None => return Err(ErrorUnauthorized("Missing Authorization header")),
            };

            let auth_header = match auth_header.to_str() {
                Ok(auth_header) => auth_header,
                Err(_) => return Err(ErrorUnauthorized("Invalid Authorization header format")),
            };

            // Check if it's a Bearer token
            if !auth_header.starts_with("Bearer ") {
                return Err(ErrorUnauthorized("Invalid Authorization format"));
            }

            // Extract the token without "Bearer " prefix
            let token = &auth_header[7..];

            // Validate JWT token
            let claims = match auth_token::validate_token(token, &auth_config) {
                Ok(claims) => claims,
                Err(_) => return Err(ErrorUnauthorized("Invalid or expired token")),
            };

            // Extract user_id from path
            let path = req.match_info();
            let target_id = path.get("user_id").unwrap_or_default();

            // Check permissions
            let is_admin = auth_token::is_admin(&claims.role);
            let is_staff = auth_token::is_staff_or_admin(&claims.role) && !is_admin;
            let is_self = claims.sub == target_id;

            let has_permission = is_self
                || (allow_admin && is_admin)
                || (allow_staff && is_staff);

            if !has_permission {
                return Err(ErrorUnauthorized("You don't have permission to access this resource"));
            }

            // Store claims in request extensions for access in handlers
            req.extensions_mut().insert(claims);

            // Call the next service
            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

// Helper trait for accessing Claims in request handlers
pub trait ClaimsFromRequest {
    fn get_claims(&self) -> Option<Claims>;
    fn user_id(&self) -> Option<String>;
    fn user_role(&self) -> Option<String>;
}

impl ClaimsFromRequest for ServiceRequest {
    fn get_claims(&self) -> Option<Claims> {
        self.extensions().get::<Claims>().cloned()
    }

    fn user_id(&self) -> Option<String> {
        self.get_claims().map(|c| c.sub.clone())
    }

    fn user_role(&self) -> Option<String> {
        self.get_claims().map(|c| c.role.clone())
    }
}

impl ClaimsFromRequest for actix_web::HttpRequest {
    fn get_claims(&self) -> Option<Claims> {
        self.extensions().get::<Claims>().cloned()
    }

    fn user_id(&self) -> Option<String> {
        self.get_claims().map(|c| c.sub.clone())
    }

    fn user_role(&self) -> Option<String> {
        self.get_claims().map(|c| c.role.clone())
    }
}