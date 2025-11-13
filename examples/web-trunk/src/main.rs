use wasm_bindgen::prelude::*;
use web_sys::wasm_bindgen::JsCast;
use web_sys::HtmlButtonElement;

fn main() {
    let document = web_sys::window().unwrap().document().unwrap();

    // get element by id button
    let button: HtmlButtonElement = document
        .get_element_by_id("button")
        .unwrap()
        .dyn_into::<HtmlButtonElement>()
        .unwrap();

    let onclick = Closure::<dyn Fn()>::new(|| {
        // Spawn dialog on main thread
        let task = rfd::AsyncFileDialog::new().pick_file();

        // Await somewhere else
        wasm_bindgen_futures::spawn_local(async {
            let file = task.await;

            let output = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("output")
                .unwrap();

            if let Some(file) = file {
                // If you care about wasm support you just read() the file
                let contents = file.read().await;

                output.set_text_content(Some(&format!("Picked file: {}, loaded {} bytes", file.file_name(), contents.len())));
            } else {
                output.set_text_content(Some("No file picked"));
            }
        });
    }).into_js_value();

    // Browsers require [user activation][mdn] to automatically show the file dialog.
    // This tests using a timer to lose transient user activation such that the file
    // dialog is not show automatically and we fall back to the popup.
    //
    // [mdn]: https://developer.mozilla.org/en-US/docs/Web/Security/User_activation
    let button_delay: HtmlButtonElement = document
        .get_element_by_id("button-delay")
        .unwrap()
        .dyn_into::<HtmlButtonElement>()
        .unwrap();

    button.set_onclick(Some(&onclick.as_ref().unchecked_ref()));

    let delay_onclick = Closure::<dyn Fn()>::new(move || {
        let window = web_sys::window().unwrap();
        window.set_timeout_with_callback_and_timeout_and_arguments_0(
            &onclick.unchecked_ref(),
            5000,
        ).unwrap();
    }).into_js_value();

    button_delay.set_onclick(Some(&delay_onclick.as_ref().unchecked_ref()));
}
