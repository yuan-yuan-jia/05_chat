use jwt_simple::prelude::*;

use crate::{error::AppError, models::User};

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISS: &str = "chat_server";
const JWT_AUD: &str = "chat_web";

pub struct EncodingKey(Ed25519KeyPair);

pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        match Ed25519KeyPair::from_pem(pem) {
            Ok(key) => Result::Ok(Self(key)),
            Err(e) => Err(AppError::CustomError(format!(
                "Failed to load encoding key: {}",
                e
            ))),
        }
    }

    pub fn sign(&self, user: impl Into<User>) -> Result<String, AppError> {
        let claims = Claims::with_custom_claims(user.into(), Duration::from_secs(JWT_DURATION));

        let claims = claims.with_issuer(JWT_ISS).with_audience(JWT_AUD);

        Ok(self
            .0
            .sign(claims)
            .map_err(|e| AppError::CustomError(format!("Failed to sign: {}", e)))?)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519PublicKey::from_pem(pem).map_err(|e| {
            AppError::CustomError(format!("Failed to load decoding key: {}", e))
        })?))
    }

    pub fn verify(&self, token: &str) -> Result<User, AppError> {
        let opts = VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISS])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUD])),
            ..Default::default()
        };

        let claims: JWTClaims<User> = self
            .0
            .verify_token(token, Some(opts))
            .map_err(|e| AppError::CustomError(format!("Failed to verify: {}", e)))?;

        Ok(claims.custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[tokio::test]
    async fn jwt_sign_verify_should_work() -> Result<()> {
        let encoding_pem = include_str!("../../fixtures/encoding.pem");
        let decoding_pem = include_str!("../../fixtures/decoding.pem");

        let ek = EncodingKey::load(encoding_pem)?;
        let dk = DecodingKey::load(decoding_pem)?;

        let user = User::new(1, 1, "Tyr Chen", "tchen@acme.org");

        let token = ek.sign(user.clone())?;
        let user2 = dk.verify(&token)?;

        assert_eq!(user, user2);
        Ok(())
    }
}
