use crate::connection::Messaging;
use crate::connection::{ClientMessage, ServerMessage};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, WebSocket};

use crate::connection::Crypto;
use crate::utils::format_wasm_traceback;
use crate::utils::formatter::log;
use crate::vdom::VirtualDom;

pub struct MessageHandler;

impl MessageHandler {
    pub fn setup_message_handler(
        ws: &WebSocket,
        window: &web_sys::Window,
        vdom_ref: Rc<RefCell<Option<VirtualDom>>>,
        crypto: &Rc<RefCell<Crypto>>,
    ) -> Result<(), crate::error::AppError> {
        // trash

        let ws_clone = ws.clone();
        let window_clone = window.clone();
        let vdom_ref = vdom_ref.clone();
        let crypto_clone = crypto.clone();

        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            let ws_clone_for_error = ws_clone.clone();
            let crypto_for_msg_handler = crypto_clone.clone();

            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let json_str: String = txt.into();
                match serde_json::from_str::<ServerMessage>(&json_str) {
                    Ok(msg) => {
                        if let Some(server_public_key_b64) = &msg.server_public_key {
                            if crypto_for_msg_handler
                                .borrow()
                                .derive_shared_secret(server_public_key_b64)
                                .is_ok()
                            {
                                let path = window_clone
                                    .location()
                                    .pathname()
                                    .unwrap_or_else(|_| "/".to_string());
                                Messaging::send_encrypted_message(
                                    &ws_clone,
                                    &ClientMessage::Navigate { path: &path },
                                    &crypto_for_msg_handler.borrow(),
                                );
                            }
                            return;
                        }

                        if let (Some(encrypted_payload_b64), Some(nonce_b64)) =
                            (&msg.encrypted_payload, &msg.nonce)
                        {
                            if let Some(decrypted_str) = crypto_for_msg_handler
                                .borrow()
                                .decrypt(encrypted_payload_b64, nonce_b64)
                            {
                                match serde_json::from_str::<ServerMessage>(&decrypted_str) {
                                    Ok(inner_msg) => {
                                        if let Some(vdom) = &mut *vdom_ref.borrow_mut() {
                                            if inner_msg.action == "render_page" {
                                                vdom.render_page(
                                                    &window_clone,
                                                    &ws_clone,
                                                    &inner_msg.content,
                                                    &inner_msg.path,
                                                    &inner_msg.css_rules,
                                                );
                                            } else if inner_msg.action == "redirect" {
                                                if let Some(url) = &inner_msg.url {
                                                    if let Some(win) = web_sys::window() {
                                                        let _ = win.location().set_href(url);
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    Err(e) => {
                                        let formatted_error = format_wasm_traceback(&e.to_string());
                                        log(&e.to_string());
                                        Messaging::send_encrypted_message(
                                            &ws_clone,
                                            &ClientMessage::ClientError {
                                                error: formatted_error,
                                            },
                                            &crypto_for_msg_handler.borrow(),
                                        );
                                    }
                                }
                            }
                            return;
                        }

                        if msg.action == "render_page" {
                            if let Some(vdom) = &mut *vdom_ref.borrow_mut() {
                                vdom.render_page(
                                    &window_clone,
                                    &ws_clone,
                                    &msg.content,
                                    &msg.path,
                                    &msg.css_rules,
                                );
                            }
                        }
                    }
                    Err(e) => {
                        let err = e.to_string();
                        let formatted_error = format_wasm_traceback(&err);
                        log(&err);
                        Messaging::send_encrypted_message(
                            &ws_clone_for_error,
                            &ClientMessage::ClientError {
                                error: formatted_error,
                            },
                            &crypto_for_msg_handler.borrow(),
                        );
                    }
                }
            }
        });

        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
        Ok(())
    }
}
