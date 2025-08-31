use super::core::ElementContent;
use crate::connection::ClientMessage;
use crate::connection::Crypto;
use crate::connection::Messaging;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use web_sys::{Document, Element, WebSocket, Window};

pub struct DomRenderer {
    crypto: Rc<RefCell<Crypto>>,
}

impl DomRenderer {
    pub fn new(crypto: Rc<RefCell<Crypto>>) -> Self {
        Self { crypto }
    }

    pub fn create_dom_element(
        &self,
        document: &Document,
        ws: &WebSocket,
        content: &ElementContent,
    ) -> Result<Element, JsValue> {
        let el = document.create_element(&content.tag)?;
        self.apply_attributes(ws, &el, &content.attributes)?;
        if let Some(text) = &content.text {
            el.set_text_content(Some(text));
        }
        for child_content in &content.children {
            let child_el = self.create_dom_element(document, ws, child_content)?;
            el.append_child(&child_el)?;
        }
        Ok(el)
    }

    pub fn apply_attributes(
        &self,
        ws: &WebSocket,
        element: &Element,
        attributes: &HashMap<String, String>,
    ) -> Result<(), JsValue> {
        for (key, value) in attributes {
            if key == "data-callback-id" {
                let callback_id = value.clone();
                let ws_clone = ws.clone();
                let crypto_clone = self.crypto.clone();
                let closure = Closure::<dyn FnMut()>::new(move || {
                    let crypto = crypto_clone.borrow();
                    Messaging::send_encrypted_message(
                        &ws_clone,
                        &ClientMessage::Callback { id: &callback_id },
                        &crypto,
                    );
                });
                let html_element = element
                    .dyn_ref::<web_sys::HtmlElement>()
                    .ok_or("Cannot adapt to HtmlElement")?;
                html_element.set_onclick(Some(closure.as_ref().unchecked_ref()));
                closure.forget();
            } else if key == "href" && element.tag_name().to_lowercase() == "a" {
                let path = value.clone();
                let ws_clone = ws.clone();
                let crypto_clone = self.crypto.clone(); // cringe
                let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
                    event.prevent_default();
                    let crypto = crypto_clone.borrow();
                    Messaging::send_encrypted_message(
                        &ws_clone,
                        &ClientMessage::Navigate { path: &path },
                        &crypto,
                    );
                });
                element.set_attribute(key, value)?;
                element
                    .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
                closure.forget();
            } else {
                element.set_attribute(key, value)?;
            }
        }
        Ok(())
    }

    pub fn apply_css_rules(
        &self,
        document: &Document,
        style_tag_id: &str,
        css_rules: &HashMap<String, HashMap<String, String>>,
    ) -> Result<(), JsValue> {
        let head = document.head().ok_or("There's no <head>. why?")?;
        if let Some(old_style) = document.get_element_by_id(style_tag_id) {
            head.remove_child(&old_style)?;
        }
        let style_tag = document
            .create_element("style")?
            .dyn_into::<web_sys::HtmlElement>()?;
        style_tag.set_id(style_tag_id);
        let mut css_string = String::new();
        for (selector, properties) in css_rules {
            css_string.push_str(&format!("{} {{\n", selector));
            for (prop, value) in properties {
                css_string.push_str(&format!("  {}: {};\n", prop, value));
            }
            css_string.push_str("}\n");
        }
        style_tag.set_text_content(Some(&css_string));
        head.append_child(&style_tag)?;
        Ok(())
    }

    pub fn update_history(window: &Window, path: &str) {
        if let Ok(history) = window.history() {
            let current_path = window.location().pathname().unwrap_or_default();
            if current_path != path {
                let _ = history.push_state_with_url(&JsValue::NULL, "", Some(path));
            }
        }
    }
}
