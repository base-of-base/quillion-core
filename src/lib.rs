mod connection;
mod error;
mod utils;
mod vdom;

pub use connection::ClientConnection;
pub use error::AppError;
pub use utils::MetaConfig;
pub use vdom::VirtualDom;

use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let config = MetaConfig::from_document()?;

    let connection =
        ClientConnection::new(&config.ws_gateway).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let vdom = VirtualDom::new(connection.get_crypto_ref());

    connection
        .start(vdom)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    Ok(())
}
