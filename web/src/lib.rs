use tcalc_core::run;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn run_web(input: String) -> String {
    match run(&input) {
        Ok(result) => result,
        Err(e) => format!("Error: {}", e),
    }
}
