mod core;
mod diff;
mod patch;
mod render;

pub use self::core::ElementContent;
use self::diff::Differ;
use self::patch::Patcher;
use self::render::DomRenderer;

use crate::connection::Crypto;
use crate::utils::log;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;
use web_sys::{WebSocket, Window};

pub struct VirtualDom {
    previous_vdom: Lazy<Mutex<Option<ElementContent>>>,
    style_tag_id: &'static str,
    crypto: Rc<RefCell<Crypto>>,
}

impl VirtualDom {
    pub fn new(crypto: Rc<RefCell<Crypto>>) -> Self {
        VirtualDom {
            previous_vdom: Lazy::new(|| Mutex::new(None)),
            style_tag_id: "quillion-dynamic-styles",
            crypto,
        }
    }

    pub fn render_page(
        &self,
        window: &Window,
        ws: &WebSocket,
        new_content: &[ElementContent],
        path_opt: &Option<String>,
        css_rules_opt: &Option<HashMap<String, HashMap<String, String>>>,
    ) {
        let document = window.document().expect("Document should exist");
        let body = document.body().expect("Document body should exist");

        let renderer = DomRenderer::new(self.crypto.clone());
        let patcher = Patcher::new(&renderer);
        let differ = Differ::new(&renderer, &patcher);

        let mut prev_vdom_guard = self.previous_vdom.lock().unwrap();
        let old_vdom_root = prev_vdom_guard.take();

        let should_full_render = old_vdom_root.is_none() || new_content.len() != 1;

        if should_full_render {
            body.set_inner_html("");
            for content in new_content {
                if let Ok(el) = renderer.create_dom_element(&document, ws, content) {
                    let _ = body.append_child(&el);
                }
            }
            *prev_vdom_guard = new_content.get(0).cloned();
        } else {
            if let (Some(old_root), Some(new_root)) = (old_vdom_root, new_content.get(0)) {
                let current_root_dom = body.first_element_child();
                if let Err(e) = differ.diff_and_patch(
                    &document,
                    ws,
                    &body,
                    current_root_dom.as_ref(),
                    Some(&old_root),
                    new_root,
                ) {
                    log(&format!("Cannot diff/patch: {:?}", e));
                }
                *prev_vdom_guard = Some(new_root.clone());
            }
        }

        if let Some(css_rules) = css_rules_opt {
            let _ = renderer.apply_css_rules(&document, self.style_tag_id, css_rules);
        }

        if let Some(path) = path_opt {
            DomRenderer::update_history(window, path);
        }
    }
}
