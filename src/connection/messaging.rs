use web_sys::WebSocket;

use crate::connection::ClientMessage;
use crate::connection::Crypto;
use crate::utils::formatter::log;

pub struct Messaging;

impl Messaging {
    pub fn send_message(ws: &WebSocket, message: &ClientMessage) {
        match serde_json::to_string(message) {
            Ok(json_str) => {
                if let Err(e) = ws.send_with_str(&json_str) {
                    log(&format!("Send error: {:?}", e));
                }
            }
            Err(e) => {
                log(&format!("Serialization error: {:?}", e));
            }
        }
    }

    pub fn send_encrypted_message(ws: &WebSocket, message: &ClientMessage, crypto: &Crypto) {
        if crypto.aes_cipher.borrow().is_none()
            && !matches!(message, ClientMessage::PublicKey { .. })
        {
            let error_msg = format!(
                "tried to send msg of type {:?} before session key has been got",
                message
            );
            log(&error_msg);
            return;
        }

        match serde_json::to_string(message) {
            Ok(json_str) => {
                if let Some((data, nonce)) = crypto.encrypt(&json_str) {
                    let encrypted_msg = ClientMessage::EncryptedMessage { data, nonce };
                    Self::send_message(ws, &encrypted_msg);
                } else {
                    if matches!(message, ClientMessage::PublicKey { .. }) {
                        Self::send_message(ws, message);
                    }
                }
            }
            Err(e) => log(&format!("Serialization error: {:?}", e)),
        }
    }
}
