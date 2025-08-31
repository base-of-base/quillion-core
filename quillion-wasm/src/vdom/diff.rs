use super::core::ElementContent;
use super::patch::Patcher;
use super::render::DomRenderer;
use std::collections::{HashMap, HashSet};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Element, WebSocket};

pub struct Differ<'a> {
    renderer: &'a DomRenderer,
    patcher: &'a Patcher<'a>,
}

impl<'a> Differ<'a> {
    pub fn new(renderer: &'a DomRenderer, patcher: &'a Patcher) -> Self {
        Self { renderer, patcher }
    }

    pub fn diff_and_patch(
        &self,
        document: &Document,
        ws: &WebSocket,
        parent_dom: &Element,
        current_dom_node: Option<&Element>,
        old_vnode: Option<&ElementContent>,
        new_vnode: &ElementContent,
    ) -> Result<Element, JsValue> {
        match (old_vnode, current_dom_node) {
            (None, _) | (_, None) => {
                let new_el = self.renderer.create_dom_element(document, ws, new_vnode)?;
                parent_dom.append_child(&new_el)?;
                Ok(new_el)
            }

            (Some(old_v), Some(current_d)) => {
                if old_v.tag != new_vnode.tag {
                    let new_el = self.renderer.create_dom_element(document, ws, new_vnode)?;
                    parent_dom.replace_child(&new_el, current_d)?;
                    Ok(new_el)
                } else {
                    self.patcher.patch_attributes(
                        current_d,
                        &old_v.attributes,
                        &new_vnode.attributes,
                        ws,
                    )?;
                    if old_v.text != new_vnode.text {
                        current_d.set_text_content(new_vnode.text.as_deref());
                    }
                    self.reconcile_children(
                        document,
                        ws,
                        current_d,
                        &old_v.children,
                        &new_vnode.children,
                    )?;
                    Ok(current_d.clone())
                }
            }
        }
    }

    fn reconcile_children(
        &self,
        document: &Document,
        ws: &WebSocket,
        parent_dom: &Element,
        old_children: &[ElementContent],
        new_children: &[ElementContent],
    ) -> Result<(), JsValue> {
        let mut old_keyed_map: HashMap<&String, (usize, &ElementContent)> = HashMap::new();
        for (i, child) in old_children.iter().enumerate() {
            if let Some(key) = &child.key {
                old_keyed_map.insert(key, (i, child));
            }
        }

        let mut new_dom_children = Vec::new();
        let mut used_old_indices = HashSet::new();

        let child_nodes = parent_dom.child_nodes();
        let mut dom_children = Vec::new();
        for i in 0..child_nodes.length() {
            if let Some(node) = child_nodes.item(i) {
                if let Ok(element) = node.dyn_into::<Element>() {
                    dom_children.push(element);
                }
            }
        }

        for new_child in new_children {
            let mut found_match = false;
            if let Some(key) = &new_child.key {
                if let Some((old_idx, old_child)) = old_keyed_map.get(key) {
                    if let Some(dom_el) = dom_children.get(*old_idx) {
                        self.diff_and_patch(
                            document,
                            ws,
                            parent_dom,
                            Some(dom_el),
                            Some(old_child),
                            new_child,
                        )?;
                        new_dom_children.push(dom_el.clone());
                        used_old_indices.insert(*old_idx);
                        found_match = true;
                    }
                }
            }
            if !found_match {
                let new_el = self.renderer.create_dom_element(document, ws, new_child)?;
                new_dom_children.push(new_el);
            }
        }

        for (i, child) in dom_children.iter().enumerate().rev() {
            if !used_old_indices.contains(&i) {
                parent_dom.remove_child(child)?;
            }
        }

        parent_dom.set_inner_html("");
        for child in new_dom_children {
            parent_dom.append_child(&child)?;
        }

        Ok(())
    }
}
