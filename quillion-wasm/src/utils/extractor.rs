use wasm_bindgen::JsCast;

pub struct EventDataExtractor;

impl EventDataExtractor {
    pub fn extract_input_data(target: &web_sys::EventTarget) -> Option<serde_json::Value> {
        if let Ok(input_element) = target.clone().dyn_into::<web_sys::HtmlInputElement>() {
            Some(serde_json::json!({
                "value": input_element.value(),
                "checked": input_element.checked()
            }))
        } else if let Ok(select_element) = target.clone().dyn_into::<web_sys::HtmlSelectElement>() {
            Some(serde_json::json!({
                "value": select_element.value(),
                "selectedIndex": select_element.selected_index()
            }))
        } else if let Ok(textarea_element) =
            target.clone().dyn_into::<web_sys::HtmlTextAreaElement>()
        {
            Some(serde_json::json!({
                "value": textarea_element.value()
            }))
        } else {
            None
        }
    }

    pub fn extract_keyboard_data(event: &web_sys::Event) -> Option<serde_json::Value> {
        if let Ok(keyboard_event) = event.clone().dyn_into::<web_sys::KeyboardEvent>() {
            Some(serde_json::json!({
                "key": keyboard_event.key(),
                "code": keyboard_event.code(),
                "ctrlKey": keyboard_event.ctrl_key(),
                "altKey": keyboard_event.alt_key(),
                "shiftKey": keyboard_event.shift_key(),
                "metaKey": keyboard_event.meta_key()
            }))
        } else {
            None
        }
    }

    pub fn extract_mouse_data(event: &web_sys::Event) -> Option<serde_json::Value> {
        if let Ok(mouse_event) = event.clone().dyn_into::<web_sys::MouseEvent>() {
            Some(serde_json::json!({
                "clientX": mouse_event.client_x(),
                "clientY": mouse_event.client_y(),
                "button": mouse_event.button(),
                "buttons": mouse_event.buttons()
            }))
        } else {
            None
        }
    }

    pub fn extract_form_data(form: &web_sys::HtmlFormElement) -> Option<serde_json::Value> {
        let mut form_data = serde_json::Map::new();

        if let Ok(elements) = form.query_selector_all("input, select, textarea") {
            for i in 0..elements.length() {
                if let Some(element) = elements.get(i) {
                    let element_clone = element.clone();

                    if let Ok(input) = element_clone.dyn_into::<web_sys::HtmlInputElement>() {
                        let name = input.name();
                        if !name.is_empty() {
                            match input.type_().as_str() {
                                "checkbox" => {
                                    form_data
                                        .insert(name, serde_json::Value::Bool(input.checked()));
                                }
                                "radio" => {
                                    if input.checked() {
                                        form_data
                                            .insert(name, serde_json::Value::String(input.value()));
                                    }
                                }
                                _ => {
                                    form_data
                                        .insert(name, serde_json::Value::String(input.value()));
                                }
                            }
                        }
                    } else {
                        let element_clone = element.clone();
                        if let Ok(select) = element_clone.dyn_into::<web_sys::HtmlSelectElement>() {
                            let name = select.name();
                            if !name.is_empty() {
                                form_data.insert(name, serde_json::Value::String(select.value()));
                            }
                        } else {
                            let element_clone = element.clone();
                            if let Ok(textarea) =
                                element_clone.dyn_into::<web_sys::HtmlTextAreaElement>()
                            {
                                let name = textarea.name();
                                if !name.is_empty() {
                                    form_data
                                        .insert(name, serde_json::Value::String(textarea.value()));
                                }
                            }
                        }
                    }
                }
            }
        }

        Some(serde_json::Value::Object(form_data))
    }
}
