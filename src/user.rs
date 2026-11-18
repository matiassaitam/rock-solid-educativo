#![allow(dead_code)]
use crate::repository::Identifiable;

/// Application users
pub struct User {
    pub username: String,
    hashed_password: String,
    pub phone: String,
}

impl User {
    /// Create a new user.
    pub fn new(username: impl Into<String>, password: &str, phone: impl Into<String>) -> Self {
        Self {
            username: username.into(),
            hashed_password: bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap(),
            phone: phone.into(),
        }
    }

    /// Check if the provided password matches the one of this user.
    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.hashed_password).unwrap_or(false)
    }
}

impl Identifiable for User {
    type Id = String;

    fn id(&self) -> &Self::Id {
        &self.username
    }
}
