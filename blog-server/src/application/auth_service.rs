use std::sync::Arc;

use argon2::{
    Argon2, password_hash::PasswordHasher, password_hash::PasswordVerifier,
    password_hash::phc::PasswordHash,
};

use crate::data::user_repository::PostgresUserRepository;
use crate::domain::error::DomainError;
use crate::domain::user::{RegisterRequest, User};
use crate::infrastructure::jwt::JwtService;

#[derive(Clone)]
pub struct AuthService {
    user_repo: Arc<PostgresUserRepository>,
    jwt_service: Arc<JwtService>,
}

impl AuthService {
    pub fn new(user_repo: Arc<PostgresUserRepository>, jwt_service: Arc<JwtService>) -> Self {
        Self {
            user_repo,
            jwt_service,
        }
    }

    pub async fn register(&self, register: RegisterRequest) -> Result<(User, String), DomainError> {
        if self
            .user_repo
            .find_by_username(&register.username)
            .await?
            .is_some()
            || self
                .user_repo
                .find_by_email(&register.email)
                .await?
                .is_some()
        {
            return Err(DomainError::UserAlreadyExists);
        }

        let password_hash = self.hash_password(&register.password)?;
        let user = self
            .user_repo
            .create_user(&register.username, &register.email, &password_hash)
            .await?;

        let token = self.jwt_service.generate_token(user.id, &user.username)?;

        Ok((user, token))
    }

    pub async fn login(
        &self,
        identifier: String,
        password: String,
    ) -> Result<(User, String), DomainError> {
        let user = if let Some(user) = self.user_repo.find_by_username(&identifier).await? {
            Some(user)
        } else {
            self.user_repo.find_by_email(&identifier).await?
        }
        .ok_or(DomainError::InvalidCredentials)?;

        self.verify_password(&password, &user.password_hash)?;

        let token = self.jwt_service.generate_token(user.id, &user.username)?;
        Ok((user, token))
    }

    fn hash_password(&self, password: &str) -> Result<String, DomainError> {
        let hash = Argon2::default()
            .hash_password(password.as_bytes())
            .map_err(|err| DomainError::Internal(err.to_string()))?
            .to_string();
        Ok(hash)
    }

    fn verify_password(&self, password: &str, password_hash: &str) -> Result<(), DomainError> {
        let hash = PasswordHash::new(password_hash)
            .map_err(|err| DomainError::Internal(err.to_string()))?;
        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .map_err(|_| DomainError::InvalidCredentials)?;
        Ok(())
    }
}
