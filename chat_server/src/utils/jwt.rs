use jwt_simple::prelude::*;

use crate::{error::AppError, User};

const JWT_DURATION: u64 = 60 * 60 * 24 * 7; // 7 days
const JWT_ISS: &str = "chat_server";
const JWT_AUD: &str = "chat_web";

#[derive(Clone)]
pub struct EncodingKey(Ed25519KeyPair);
#[derive(Debug, Clone)]
pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(EncodingKey(Ed25519KeyPair::from_pem(pem)?))
    }

    pub fn sign(&self, user: User) -> Result<String, AppError> {
        let mut claims = Claims::with_custom_claims(user, Duration::from_secs(JWT_DURATION));
        claims = claims.with_issuer(JWT_ISS).with_audience(JWT_AUD);
        Ok(self.0.sign(claims)?)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(DecodingKey(Ed25519PublicKey::from_pem(pem)?))
    }

    #[allow(unused, clippy::field_reassign_with_default)]
    pub fn verify(&self, token: &str) -> Result<User, AppError> {
        let mut options = VerificationOptions::default();
        options.allowed_issuers = Some(HashSet::from_strings(&[JWT_ISS]));
        options.allowed_audiences = Some(HashSet::from_strings(&[JWT_AUD]));

        let claims = self.0.verify_token::<User>(token, Some(options))?;
        Ok(claims.custom)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::error::AppError;

    #[test]
    fn user_sign_verify_should_work() -> Result<(), AppError> {
        let ek = EncodingKey::load(include_str!("../../fixtures/encoding.pem"))?;
        let pk = DecodingKey::load(include_str!("../../fixtures/decoding.pem"))?;

        let user = User::new(1, "tom", "tom@123.com");

        let token = ek.sign(user.clone())?;
        let user2 = pk.verify(&token)?;

        assert_eq!(user, user2);

        Ok(())
    }
}
