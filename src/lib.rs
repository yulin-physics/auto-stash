use api::Api;

use storage::get_from_chrome_storage;
use storage::save_to_chrome_storage;

use wasm_bindgen::prelude::*;
use web_sys::window;

pub mod api;
pub mod storage;

#[wasm_bindgen(start)]
pub async fn init() -> Result<(), JsError> {
    // console_log::init_with_level(log::Level::Info).expect("can't initialize logger");
    let window = window().ok_or(JsError::new("no window"))?;
    let mut api = Api::new(window);
    api.start()
        .await
        .map_err(|e| JsError::new(&e.to_string()))?;

    Ok(())
}
