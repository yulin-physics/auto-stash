use anyhow::Result;
use std::collections::HashMap;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, prelude::Closure};
use wasm_bindgen::{JsError, JsValue};
use web_sys::{
    Document, Element, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement, Window,
};

use crate::{get_from_chrome_storage, save_to_chrome_storage, storage::FormField};

const ENABLED: &str = "enabled";
type FormFields = HashMap<String, String>;

#[wasm_bindgen]
pub struct Api {
    window: Window,
    url: String,
    fields: FormFields,
}

#[wasm_bindgen]
impl Api {
    #[wasm_bindgen(js_name = "start")]
    pub async fn js_start(&mut self) -> Result<(), JsError> {
        self.start().await.map_err(|e| JsError::new(&e.to_string()))
    }
}

impl Api {
    pub fn new(window: Window) -> Self {
        Self {
            window,
            url: String::default(),
            fields: HashMap::new(),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        // Check if auto save is enabled
        let enabled = get_key_from_storage(ENABLED).await?;
        if !enabled.as_bool().unwrap_or(false) {
            return Ok(());
        }

        // Data save keyed by url (excludes query params)
        let location = self.window.location();
        let origin = location
            .origin()
            .map_err(|e| Error::FailedToFindOrigin(e.as_string().unwrap_or_default()))?;
        let pathname = location
            .pathname()
            .map_err(|e| Error::FailedToFindPathname(e.as_string().unwrap_or_default()))?;

        self.url = format!("{origin}{pathname}");

        let saved_data = get_key_from_storage(&self.url).await?;
        if !saved_data.is_null() && !saved_data.is_undefined() {
            self.fields = serde_wasm_bindgen::from_value(saved_data)
                .map_err(|e| Error::FailedToFindPathname(e.to_string()))?;
        }

        self.restore().await?;

        self.save().await?;

        Ok(())
    }

    async fn restore(&self) -> Result<()> {
        for (key, val) in self.fields.iter() {
            if let Some(el) = self
                .document()?
                .query_selector(&format!("[id='{}'], #{}", key, key))
                .map_err(|e| Error::FailedToFindElement(e.as_string().unwrap_or_default()))?
            {
                if let Some(input) = el.dyn_ref::<HtmlInputElement>() {
                    if input.value().is_empty() {
                        input.set_value(val);
                    }
                } else if let Some(select) = el.dyn_ref::<HtmlSelectElement>() {
                    select.set_value(val);
                } else if let Some(textarea) = el.dyn_ref::<HtmlTextAreaElement>() {
                    if textarea.value().is_empty() {
                        textarea.set_value(val);
                    }
                }
            }
        }

        Ok(())
    }

    async fn save(&self) -> Result<()> {
        // Query all inputs, selects, textareas
        let inputs = self
            .document()?
            .query_selector_all("input, select, textarea")
            .map_err(|e| Error::CannotQueryInputs(e.as_string().unwrap_or_default()))?;

        let length = inputs.length();

        // Attach event listeners to save on input/change
        for i in 0..length {
            let node = inputs.item(i).unwrap();

            if let Some(el) = node.dyn_ref::<web_sys::Element>() {
                attach_listener(self.url.clone(), el).map_err(|e| {
                    Error::FailedToAttachListener(e.as_string().unwrap_or_default())
                })?;
            }
        }

        Ok(())
    }

    fn document(&self) -> Result<Document> {
        Ok(self.window.document().ok_or(Error::NoDocument)?)
    }
}

async fn get_key_from_storage(key: &str) -> Result<JsValue> {
    let js_object = get_from_chrome_storage(key).await;

    Ok(js_sys::Reflect::get(&js_object, &JsValue::from_str(key))
        .map_err(|e| Error::FailedToGetKeyEnabled(e.as_string().unwrap_or_default()))?)
}

// Attach event listener to input/select/textarea to save updated value
fn attach_listener(url: String, el: &Element) -> Result<(), JsValue> {
    let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let target = event.target().unwrap();
        let element = target.dyn_into::<Element>().unwrap();

        if let Some(name) = get_element_name(&element) {
            let value = get_element_value(&element);
            let field = FormField { name, value };
            let js_val = serde_wasm_bindgen::to_value(&field).unwrap();

            save_to_chrome_storage(&url, &js_val);
        }
    }) as Box<dyn FnMut(_)>);

    el.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref())?;
    el.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;

    closure.forget();

    Ok(())
}

// Get name or id attribute for element
fn get_element_name(el: &Element) -> Option<String> {
    // if let Some(name) = el.get_attribute("name") {
    //     if !name.is_empty() {
    //         return Some(name);
    //     }
    // }
    el.get_attribute("id")
}

// Get value depending on element type
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

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to get key from storage: {0}")]
    FailedToGetKeyEnabled(String),

    #[error("failed to find origin: {0}")]
    FailedToFindOrigin(String),

    #[error("failed to find pathname: {0}")]
    FailedToFindPathname(String),

    #[error("failed to parse form data: {0}")]
    FailedToParseFormData(String),

    #[error("no document")]
    NoDocument,

    #[error("cannot query inputs: {0}")]
    CannotQueryInputs(String),

    #[error("failed to attach listener: {0}")]
    FailedToAttachListener(String),

    #[error("failed to find element: {0}")]
    FailedToFindElement(String),
}
