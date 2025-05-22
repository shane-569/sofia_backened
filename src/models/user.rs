use serde::{Serialize, Deserialize};
use mongodb::bson::{oid::ObjectId};
use bcrypt::{hash, DEFAULT_COST};
use crate::models::role::Role;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub password: String,
    pub role: Role,
}

impl User {
    pub fn new(username: String, raw_password: String,role: Role) -> Self {
        let hashed = hash(raw_password, DEFAULT_COST).unwrap();
        User {
            id: None,
            username,
            password: hashed,
            role,
        }
    }
}