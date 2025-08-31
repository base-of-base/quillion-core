use super::render::DomRenderer;
use std::collections::HashMap;
use wasm_bindgen::JsValue;
use web_sys::{Element, WebSocket};

pub struct Patcher<'a> {
    renderer: &'a DomRenderer,
}

impl<'a> Patcher<'a> {
    pub fn new(renderer: &'a DomRenderer) -> Self {
        Self { renderer }
    }

    pub fn patch_attributes(
        &self,
        element: &Element,
        old_attrs: &HashMap<String, String>,
        new_attrs: &HashMap<String, String>,
        ws: &WebSocket,
    ) -> Result<(), JsValue> {
        for (key, _) in old_attrs {
            if !new_attrs.contains_key(key) {
                element.remove_attribute(key)?;
            }
        }

        for (key, value) in new_attrs {
            if old_attrs.get(key) != Some(value) {
                self.renderer.apply_attributes(
                    ws,
                    element,
                    &HashMap::from([(key.clone(), value.clone())]),
                )?;
            }
        }
        Ok(())
    }
}
