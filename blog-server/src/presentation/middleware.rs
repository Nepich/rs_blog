use std::sync::Arc;

use actix_web::{Error, HttpMessage, dev::ServiceRequest, web};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use actix_web_httpauth::headers::www_authenticate::bearer::Bearer;

use crate::infrastructure::jwt::JwtService;

#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: i64,
}

fn auth_error() -> AuthenticationError<Bearer> {
    AuthenticationError::new(Bearer::build().realm("Restricted").finish())
}

pub async fn jwt_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_service = match req.app_data::<web::Data<Arc<JwtService>>>() {
        Some(data) => data.get_ref().clone(),
        None => return Err((auth_error().into(), req)),
    };

    let token = credentials.token();
    let claims = match jwt_service.verify_token(token) {
        Ok(claims) => claims,
        Err(_) => return Err((auth_error().into(), req)),
    };

    let user = AuthenticatedUser {
        user_id: claims.user_id,
    };

    req.extensions_mut().insert(user);
    Ok(req)
}
