use std::cell::RefCell;
use std::rc::Rc;
use web_sys::{WebSocket, Window};

use crate::connection::Crypto;
use crate::connection::EventHandler;
use crate::error::AppError;
use crate::vdom::VirtualDom;

#[derive(Clone)]
pub struct ClientConnection {
    pub ws: WebSocket,
    pub window: Window,
    pub vdom: Rc<RefCell<Option<VirtualDom>>>,
    pub crypto: Rc<RefCell<Crypto>>,
    pub ws_gateway: String, 
}

impl ClientConnection {
    pub fn new(server_url: &str) -> Result<Self, AppError> {
        let window = web_sys::window().ok_or(AppError::WindowNotFound)?;
        let ws = WebSocket::new(server_url)
            .map_err(|e| AppError::WebSocketError(e.as_string().unwrap_or_default()))?;

        Ok(Self {
            ws,
            window,
            vdom: Rc::new(RefCell::new(None)),
            crypto: Rc::new(RefCell::new(Crypto::new())),
            ws_gateway: server_url.to_string(),
        })
    }

    pub fn start(&self, vdom: VirtualDom) -> Result<(), AppError> {
        *self.vdom.borrow_mut() = Some(vdom);
        EventHandler::setup_handlers(self)?;
        Ok(())
    }

    pub fn get_crypto_ref(&self) -> Rc<RefCell<Crypto>> {
        self.crypto.clone()
    }
}
