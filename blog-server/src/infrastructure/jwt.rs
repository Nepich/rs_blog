use chrono::{Duration, Utc};
use jsonwebtoken::{
    DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::Error as JwtError,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub exp: usize,
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(secret: &str) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        Self {
            encoding_key,
            decoding_key,
        }
    }

    pub fn generate_token(&self, user_id: i64, username: &str) -> Result<String, JwtError> {
        let exp = (Utc::now() + Duration::hours(24)).timestamp() as usize;
        let claims = Claims {
            user_id,
            username: username.to_string(),
            exp,
        };
        encode(&Header::default(), &claims, &self.encoding_key)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, JwtError> {
        let token_data = decode::<Claims>(token, &self.decoding_key, &Validation::default())?;
        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_service_generate_and_verify_token() {
        let service = JwtService::new("test-secret");
        let token = service.generate_token(1, "alice").expect("failed to generate token");
        let claims = service.verify_token(&token).expect("failed to verify token");

        assert_eq!(claims.user_id, 1);
        assert_eq!(claims.username, "alice");
    }

    #[test]
    fn test_jwt_service_rejects_invalid_token() {
        let service = JwtService::new("test-secret");
        assert!(service.verify_token("invalid.token.value").is_err());
    }
}
