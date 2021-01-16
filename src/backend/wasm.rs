// use crate::FileDialog;
use std::path::PathBuf;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element};

use web_sys::{HtmlButtonElement, HtmlInputElement};

use crate::{FileDialog, FileHandle};

pub struct WasmDialog {
    overlay: Element,
    card: Element,
    input: HtmlInputElement,
    button: HtmlButtonElement,

    style: Element,
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

impl WasmDialog {
    pub fn new(opt: &FileDialog) -> Self {
        let window = web_sys::window().expect("Window not found");
        let document = window.document().expect("Document not found");

        let overlay = document.create_element("div").unwrap();
        overlay.set_id("rfd-overlay");

        let card = {
            let card = document.create_element("div").unwrap();
            card.set_id("rfd-card");
            overlay.append_child(&card).unwrap();

            card
        };

        let input = {
            let input_el = document.create_element("input").unwrap();
            let input: HtmlInputElement = wasm_bindgen::JsCast::dyn_into(input_el).unwrap();

            input.set_id("rfd-input");
            input.set_type("file");

            let mut accept: Vec<String> = Vec::new();

            for filter in opt.filters.iter() {
                accept.append(&mut filter.extensions.to_vec());
            }

            input.set_accept(&accept.join(","));

            card.append_child(&input).unwrap();
            input
        };

        let button = {
            let btn_el = document.create_element("button").unwrap();
            let btn: HtmlButtonElement = wasm_bindgen::JsCast::dyn_into(btn_el).unwrap();

            btn.set_id("rfd-button");
            btn.set_inner_text("Ok");

            card.append_child(&btn).unwrap();
            btn
        };

        let style = document.create_element("style").unwrap();
        style.set_inner_html(include_str!("./wasm/style.css"));
        overlay.append_child(&style).unwrap();

        Self {
            overlay,
            card,
            button,
            input,

            style,
        }
    }

    async fn show(&self) {
        let window = web_sys::window().expect("Window not found");
        let document = window.document().expect("Document not found");
        let body = document.body().expect("document should have a body");

        let overlay = self.overlay.clone();
        let button = self.button.clone();

        let promise = js_sys::Promise::new(&mut move |res, _rej| {
            let closure = Closure::wrap(Box::new(move || {
                res.call0(&JsValue::undefined()).unwrap();
            }) as Box<dyn FnMut()>);

            button.set_onclick(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
            body.append_child(&overlay).ok();
        });
        let future = wasm_bindgen_futures::JsFuture::from(promise);
        future.await.unwrap();
    }

    fn get_results(&self) -> Option<Vec<FileHandle>> {
        if let Some(files) = self.input.files() {
            let len = files.length();
            if len > 0 {
                let mut file_handles = Vec::new();
                for id in 0..len {
                    let file = files.get(id).unwrap();
                    file_handles.push(FileHandle::wrap(file));
                }
                Some(file_handles)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_result(&self) -> Option<FileHandle> {
        let mut files = self.get_results();
        files.and_then(|mut f| f.pop())
    }

    async fn pick_files(self) -> Option<Vec<FileHandle>> {
        self.input.set_multiple(true);

        self.show().await;

        self.get_results()
    }

    async fn pick_file(self) -> Option<FileHandle> {
        self.input.set_multiple(false);

        self.show().await;

        self.get_result()
    }
}

impl Drop for WasmDialog {
    fn drop(&mut self) {
        self.button.remove();
        self.input.remove();
        self.card.remove();

        self.style.remove();
        self.overlay.remove();
    }
}

use std::future::Future;

pub fn pick_file_async(opt: FileDialog) -> impl Future<Output = Option<FileHandle>> {
    let dialog = WasmDialog::new(&opt);
    dialog.pick_file()
}

// pub fn save_file_async(opt: FileDialog) -> impl Future<Output = Option<FileHandle>> {
//     let dialog = WasmDialog::new(&opt);
//     dialog.pick_file()
// }

// pub fn pick_folder_async(opt: FileDialog) -> impl Future<Output = Option<FileHandle>> {
//     let dialog = WasmDialog::new(&opt);
//     dialog.pick_file()
// }

pub fn pick_files_async(opt: FileDialog) -> impl Future<Output = Option<Vec<FileHandle>>> {
    let dialog = WasmDialog::new(&opt);
    dialog.pick_files()
}
