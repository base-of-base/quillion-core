use super::core::ElementContent;
use crate::connection::{ClientMessage, Crypto, Messaging};
use crate::utils::EventDataExtractor;
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
            match key.as_str() {
                key if key.starts_with("on") && key.len() > 2 => {
                    self.set_event_handler(ws, element, key, value)?;
                }
                "data-callback-id" => {
                    self.set_callback_handler(ws, element, value)?;
                }
                "href" if element.tag_name().to_lowercase() == "a" => {
                    self.set_link_handler(ws, element, value)?;
                }
                _ => {
                    element.set_attribute(key, value)?;
                }
            }
        }
        Ok(())
    }

    fn set_event_handler(
        &self,
        ws: &WebSocket,
        element: &Element,
        event_name: &str,
        callback_id: &str,
    ) -> Result<(), JsValue> {
        let event_type = event_name[2..].to_string();
        let event_type_clone = event_type.clone();
        let callback_id = callback_id.to_string();
        let ws_clone = ws.clone();
        let crypto_clone = self.crypto.clone();

        let closure = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
            Self::handle_event(&event, &event_type_clone);

            let event_data = Self::extract_event_data(&event, &event_type_clone);
            let crypto = crypto_clone.borrow();

            Messaging::send_encrypted_message(
                &ws_clone,
                &ClientMessage::EventCallback {
                    id: &callback_id,
                    event_type: event_type_clone.clone(),
                    event_data: event_data.unwrap_or_default(),
                },
                &crypto,
            );
        });

        element.add_event_listener_with_callback(&event_type, closure.as_ref().unchecked_ref())?;
        closure.forget();

        Ok(())
    }

    fn handle_event(event: &web_sys::Event, event_type: &str) {
        match event_type {
            "submit" => {
                event.prevent_default();
            }
            "click" => {
                if let Some(target) = event.target() {
                    if let Ok(element) = target.dyn_into::<Element>() {
                        if element.tag_name().to_lowercase() == "a" {
                            event.prevent_default();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn extract_event_data(event: &web_sys::Event, event_type: &str) -> Option<String> {
        let data = match event_type {
            "input" | "change" => event
                .target()
                .and_then(|target| EventDataExtractor::extract_input_data(&target)),
            "keydown" | "keyup" | "keypress" => EventDataExtractor::extract_keyboard_data(event),
            "click" | "dblclick" | "mousedown" | "mouseup" | "mousemove" => {
                EventDataExtractor::extract_mouse_data(event)
            }
            "submit" => event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlFormElement>().ok())
                .and_then(|form| EventDataExtractor::extract_form_data(&form)),
            _ => None,
        };
        data.map(|d| d.to_string())
    }

    fn set_callback_handler(
        &self,
        ws: &WebSocket,
        element: &Element,
        callback_id: &str,
    ) -> Result<(), JsValue> {
        let callback_id = callback_id.to_string();
        let ws_clone = ws.clone();
        let crypto_clone = self.crypto.clone();

        let closure = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
            event.prevent_default();

            let crypto = crypto_clone.borrow();
            Messaging::send_encrypted_message(
                &ws_clone,
                &ClientMessage::Callback { id: &callback_id },
                &crypto,
            );
        });

        element.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();

        Ok(())
    }

    fn set_link_handler(
        &self,
        ws: &WebSocket,
        element: &Element,
        path: &str,
    ) -> Result<(), JsValue> {
        element.set_attribute("href", path)?;

        let path = path.to_string();
        let ws_clone = ws.clone();
        let crypto_clone = self.crypto.clone();

        let closure = Closure::<dyn FnMut(web_sys::Event)>::new(move |event: web_sys::Event| {
            event.prevent_default();

            let crypto = crypto_clone.borrow();
            Messaging::send_encrypted_message(
                &ws_clone,
                &ClientMessage::Navigate { path: &path },
                &crypto,
            );
        });

        element.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();

        Ok(())
    }

    pub fn apply_css_rules(
        &self,
        document: &Document,
        style_tag_id: &str,
        css_rules: &HashMap<String, HashMap<String, String>>,
    ) -> Result<(), JsValue> {
        let head = document.head().ok_or("No <head> element found")?;

        if let Some(old_style) = document.get_element_by_id(style_tag_id) {
            let _ = head.remove_child(&old_style);
        }

        let style_tag = document
            .create_element("style")?
            .dyn_into::<web_sys::HtmlElement>()?;
        style_tag.set_id(style_tag_id);

        let css_string = css_rules
            .iter()
            .map(|(selector, properties)| {
                let props = properties
                    .iter()
                    .map(|(prop, value)| format!("  {}: {};", prop, value))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{} {{\n{}\n}}", selector, props)
            })
            .collect::<Vec<_>>()
            .join("\n");

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
