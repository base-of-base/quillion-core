use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::vdom::ElementContent;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerMessage {
    pub action: String,
    #[serde(default)]
    pub content: Vec<ElementContent>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub css_rules: Option<HashMap<String, HashMap<String, String>>>,
    #[serde(default)]
    pub server_public_key: Option<String>,
    #[serde(default)]
    pub encrypted_payload: Option<String>,
    #[serde(default)]
    pub nonce: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}
