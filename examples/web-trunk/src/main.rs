use wasm_bindgen::prelude::*;
use web_sys::wasm_bindgen::JsCast;
use web_sys::HtmlButtonElement;

fn main() {
    // get element by id button
    let button: HtmlButtonElement = web_sys::window()
        .unwrap()
        .document()
        .unwrap()
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

    button.set_onclick(Some(&onclick.as_ref().unchecked_ref()));
}
