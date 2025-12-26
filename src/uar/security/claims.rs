use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaims {
    pub sub: String, // User ID (Subject)
    pub name: Option<String>,
    pub roles: Option<Vec<String>>,
    pub exp: usize, // Expiration time (UNIX timestamp)
}

#[derive(Clone, Debug)]
pub struct UserContext {
    pub user_id: String,
    pub claims: UserClaims,
}
