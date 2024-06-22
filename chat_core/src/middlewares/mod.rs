mod auth;
mod request_id;
mod server_time;

use crate::User;

pub trait TokenVerifier {
    type Error;
    fn verify(&self, token: &str) -> Result<User, Self::Error>;
}

pub const REQUEST_ID_HEADER: &str = "x-request-id";
pub const SERVER_TIME_HEADER: &str = "x-server-time";
pub use auth::verify_token;
pub use request_id::set_request_id;
pub use server_time::ServerTimeLayer;
