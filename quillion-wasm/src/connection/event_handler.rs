use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{ErrorEvent, PopStateEvent, WebSocket, Window};

use crate::connection::ClientConnection;
use crate::connection::Crypto;
use crate::connection::{ClientMessage, MessageHandler, Messaging};
use crate::error::AppError;
use crate::utils::log;
use std::cell::RefCell;
use std::rc::Rc;

pub struct EventHandler;

impl EventHandler {
    pub fn setup_handlers(conn: &ClientConnection) -> Result<(), AppError> {
        MessageHandler::setup_message_handler(
            &conn.ws,
            &conn.window,
            conn.vdom.clone(),
            &conn.crypto,
        )?;
        Self::setup_popstate_handler(&conn.ws, &conn.window, &conn.crypto)?;
        Self::setup_error_handler(&conn.ws)?;
        Self::setup_open_handler(&conn.ws, &conn.crypto)?;
        Ok(())
    }

    fn setup_popstate_handler(
        ws: &WebSocket,
        window: &Window,
        crypto: &Rc<RefCell<Crypto>>,
    ) -> Result<(), AppError> {
        let ws_clone = ws.clone();
        let window_clone = window.clone();
        let crypto_clone = crypto.clone();

        let onpopstate_callback = Closure::<dyn FnMut(_)>::new(move |_: PopStateEvent| {
            let path = window_clone
                .location()
                .pathname()
                .unwrap_or_else(|_| "/".to_string());
            Messaging::send_encrypted_message(
                &ws_clone,
                &ClientMessage::Navigate { path: &path },
                &crypto_clone.borrow(),
            );
        });

        window.set_onpopstate(Some(onpopstate_callback.as_ref().unchecked_ref()));
        onpopstate_callback.forget();
        Ok(())
    }

    fn setup_error_handler(ws: &WebSocket) -> Result<(), AppError> {
        let ws = ws.clone();
        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
            log(&format!("WebSocket error: {:?}", e));
        });
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
        Ok(())
    }

    fn setup_open_handler(ws: &WebSocket, crypto: &Rc<RefCell<Crypto>>) -> Result<(), AppError> {
        let ws_clone = ws.clone();
        let crypto_clone = crypto.clone();
        let public_key_b64 = crypto_clone.borrow().public_key_b64();

        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            Messaging::send_message(
                &ws_clone,
                &ClientMessage::PublicKey {
                    key: public_key_b64.clone(),
                },
            );
        });

        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
        Ok(())
    }
}
