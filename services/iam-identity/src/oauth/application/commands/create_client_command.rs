use cuba_cqrs_core::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateClientCommand {
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub grant_types: Vec<String>,
    pub scopes: Vec<String>,
    pub client_secret: Option<String>,
    pub public_client: bool,
    pub tenant_id: String,
}

impl Command for CreateClientCommand {
    type Result = (String, Option<String>);
}
