use wasm_bindgen::JsValue;
use web_sys::{Document, window};

#[derive(Debug, Clone)]
pub struct MetaConfig {
    pub ws_gateway: String,
}

impl MetaConfig {
    pub fn from_document() -> Result<Self, JsValue> {
        let document = Self::get_document()?;

        let is_dev = Self::get_meta_content(&document, "is-dev").is_some();
        let ws_gateway_val = Self::get_meta_content(&document, "ws-gateway");

        let ws_gateway = if is_dev && ws_gateway_val.is_none() {
            "ws://localhost:1337".to_string()
        } else if let Some(gateway) = ws_gateway_val {
            gateway
        } else {
            Self::build_ws_url_from_location(1337)?
        };

        Ok(Self { ws_gateway })
    }

    fn get_meta_content(document: &Document, name: &str) -> Option<String> {
        let selector = format!("meta[name='{}']", name);
        document
            .query_selector(&selector)
            .ok()
            .flatten()
            .and_then(|meta| meta.get_attribute("content"))
            .filter(|content| !content.is_empty())
    }

    fn get_document() -> Result<Document, JsValue> {
        window()
            .ok_or_else(|| JsValue::from_str("No window found"))
            .and_then(|w| {
                w.document()
                    .ok_or_else(|| JsValue::from_str("No document found"))
            })
    }

    fn build_ws_url_from_location(port: u16) -> Result<String, JsValue> {
        let window = window().ok_or_else(|| JsValue::from_str("No window"))?;
        let location = window.location();
        let protocol = location.protocol()?;
        let hostname = location.hostname()?;

        let ws_protocol = if protocol == "https:" { "wss:" } else { "ws:" };
        Ok(format!("{}//{}:{}", ws_protocol, hostname, port))
    }
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self {
            ws_gateway: "ws://localhost:1337".into(),
        }
    }
}
