//! User DTO

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::shared::domain::entities::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub email: String,
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub tenant_id: String,
    pub role_ids: Vec<String>,
    pub status: String,
    pub language: String,
    pub timezone: String,
    pub two_factor_enabled: bool,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<User> for UserDto {
    fn from(user: User) -> Self {
        Self {
            id: user.id.0.to_string(),
            username: user.username.as_str().to_string(),
            email: user.email.as_str().to_string(),
            display_name: user.display_name,
            phone: user.phone,
            avatar_url: user.avatar_url,
            tenant_id: user.tenant_id.0.to_string(),
            role_ids: user.role_ids,
            status: format!("{:?}", user.status),
            language: user.language,
            timezone: user.timezone,
            two_factor_enabled: user.two_factor_enabled,
            last_login_at: user.last_login_at,
        }
    }
}
