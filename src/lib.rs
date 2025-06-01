use js_sys::{Array, JsString, Object};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{
    Document, Element, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, window,
};

// prompt to start auto-save
// TODO: clear storage if form submitted and successful - prompt to clear storage
// key storage by site domain

#[wasm_bindgen(module = "/js_helpers.js")]
extern "C" {
    fn save_to_chrome_storage(key: &str, value: &JsValue);
    fn get_from_chrome_storage(key: &str) -> js_sys::Promise;
    fn clear_chrome_storage(key: &str);
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Storage {
    forms: Vec<Form>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Form {
    key: String,
    value: Vec<FormField>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FormField {
    id: String,
    value: String,
}

#[wasm_bindgen(start)]
pub async fn init() -> Result<(), JsValue> {
    let window = window().ok_or("no window")?;
    let document = window.document().ok_or("no document")?;
    log::info!("in ");
    // Check if form was submitted
    let submitted_js =
        wasm_bindgen_futures::JsFuture::from(get_from_chrome_storage("form_submitted")).await?;

    log::info!("{submitted_js:?} ");
    let submitted_obj = js_sys::Reflect::get(&submitted_js, &JsValue::from_str("form_submitted"))?;
    if submitted_obj.as_bool().unwrap_or(false) {
        // Form was submitted, skip restoring fields
        return Ok(());
    }

    // Restore saved form fields
    let saved_js =
        wasm_bindgen_futures::JsFuture::from(get_from_chrome_storage("formData")).await?;
    log::info!("{saved_js:?} ");
    let saved_data = js_sys::Reflect::get(&saved_js, &JsValue::from_str("formData"))?;
    if saved_data.is_undefined() || saved_data.is_null() {
        // No saved data, skip restore
    } else {
        if let Some(fields) = saved_data.dyn_ref::<js_sys::Object>() {
            let keys = js_sys::Object::keys(fields);
            for i in 0..keys.length() {
                let key = keys.get(i).as_string().unwrap();
                let val = js_sys::Reflect::get(fields, &JsValue::from_str(&key))
                    .unwrap()
                    .as_string()
                    .unwrap();

                // Find element and set value
                // let el = document.get_element_by_id(&key).unwrap();
                // log::info!("this {el:?} {val:?}");
                // el.set_attribute("value", &val).unwrap();

                if let Some(el) = document.query_selector(&format!("[id='{}'], #{}", key, key))? {
                    if let Some(input) = el.dyn_ref::<HtmlInputElement>() {
                        input.set_value(&val);
                    } else if let Some(select) = el.dyn_ref::<HtmlSelectElement>() {
                        select.set_value(&val);
                    } else if let Some(textarea) = el.dyn_ref::<HtmlTextAreaElement>() {
                        textarea.set_value(&val);
                    }
                }
            }
        }
    }

    // Query all inputs, selects, textareas
    let inputs = document.query_selector_all("input, select, textarea")?;
    log::info!("{inputs:?} ");

    let length = inputs.length();

    // Attach event listeners to save on input/change
    for i in 0..length {
        let node = inputs.item(i).unwrap();

        if let Some(el) = node.dyn_ref::<web_sys::Element>() {
            attach_listener(&el)?;
        }
    }

    // Attach submit listener to all forms
    let forms = document.query_selector_all("form")?;
    for i in 0..forms.length() {
        if let Some(node) = forms.item(i) {
            if let Some(el) = node.dyn_ref::<web_sys::Element>() {
                attach_submit_listener(&el)?;
            }
        }
    }

    Ok(())
}

// Helper: Get name or id attribute for element
fn get_element_name(el: &Element) -> Option<String> {
    // if let Some(name) = el.get_attribute("name") {
    //     if !name.is_empty() {
    //         return Some(name);
    //     }
    // }
    el.get_attribute("id")
}

// Helper: Get value depending on element type
fn get_element_value(el: &Element) -> String {
    if let Some(input) = el.dyn_ref::<HtmlInputElement>() {
        input.value()
    } else if let Some(select) = el.dyn_ref::<HtmlSelectElement>() {
        select.value()
    } else if let Some(textarea) = el.dyn_ref::<HtmlTextAreaElement>() {
        textarea.value()
    } else {
        "".to_string()
    }
}

// Attach event listener to input/select/textarea to save updated value
fn attach_listener(el: &Element) -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let target = event.target().unwrap();
        let element = target.dyn_into::<Element>().unwrap();

        if let Some(name) = get_element_name(&element) {
            let value = get_element_value(&element);
            let field = FormField { id: name, value };
            let js_val = serde_wasm_bindgen::to_value(&field).unwrap();

            save_to_chrome_storage("form_fields", &js_val);
        }
    }) as Box<dyn FnMut(_)>);

    el.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
    el.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;

    closure.forget();

    Ok(())
}

// Attach submit event listener to form to mark form as submitted
fn attach_submit_listener(form: &Element) -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        // Mark form submitted
        save_to_chrome_storage("form_submitted", &JsValue::from_bool(true));
        // Clear saved form data
        clear_chrome_storage("formData");
    }) as Box<dyn FnMut(_)>);

    form.add_event_listener_with_callback("submit", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}
