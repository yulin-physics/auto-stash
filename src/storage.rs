use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

// #[derive(Serialize, Deserialize, Clone)]
// pub struct Storage {
//     pub enabled: String,
//     pub forms: Vec<Form>,
// }

// #[derive(Serialize, Deserialize, Clone)]
// pub struct Form {
//     pub key: String,
//     pub value: Vec<FormField>,
// }

#[derive(Serialize, Deserialize, Clone)]
pub struct FormField {
    pub name: String,
    pub value: String,
}

#[wasm_bindgen(module = "/js_helpers.js")]
extern "C" {
    pub(crate) fn save_to_chrome_storage(key: &str, value: &JsValue);
    pub(crate) fn get_from_chrome_storage(key: &str) -> js_sys::Promise;
    pub(crate) fn clear_chrome_storage(key: &str);
}
