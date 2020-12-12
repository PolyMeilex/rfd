use crate::DialogOptions;
use std::path::PathBuf;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Document, Element};

use web_sys::{HtmlButtonElement, HtmlInputElement};

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

pub struct Dialog {
    overlay: Element,
    card: Element,
    input: HtmlInputElement,
    button: HtmlButtonElement,

    style: Element,

    closure: Option<Closure<dyn FnMut()>>,
}

impl Dialog {
    pub fn new(document: &Document) -> Self {
        let overlay = document.create_element("div").unwrap();
        overlay.set_id("rfd-overlay");

        let card = {
            let card = document.create_element("div").unwrap();
            card.set_id("rfd-card");
            overlay.append_child(&card);

            card
        };

        let input = {
            let input_el = document.create_element("input").unwrap();
            let input: HtmlInputElement = wasm_bindgen::JsCast::dyn_into(input_el).unwrap();

            input.set_id("rfd-input");
            input.set_type("file");

            card.append_child(&input);
            input
        };

        let button = {
            let btn_el = document.create_element("button").unwrap();
            let btn: HtmlButtonElement = wasm_bindgen::JsCast::dyn_into(btn_el).unwrap();

            btn.set_id("rfd-button");
            btn.set_inner_text("Ok");

            card.append_child(&btn);
            btn
        };

        let style = document.create_element("style").unwrap();
        style.set_inner_html(include_str!("./wasm/style.css"));
        overlay.append_child(&style);

        Self {
            overlay,
            card,
            button,
            input,

            style,

            closure: None,
        }
    }

    pub fn open<F: Fn() + 'static>(&mut self, body: &Element, cb: F) {
        let overlay = self.overlay.clone();
        let input = self.input.clone();
        let closure = Closure::wrap(Box::new(move || {
            // let files = Vec::new();

            if let Some(files) = input.files() {
                for id in 0..(files.length()) {
                    let file = files.get(id).unwrap();
                    let file_reader = web_sys::FileReader::new().unwrap();

                    let fr = file_reader.clone();
                    let closure = Closure::wrap(Box::new(move || {
                        let res = fr.result().unwrap();

                        let buffer: js_sys::Uint8Array = js_sys::Uint8Array::new(&res);
                        let mut vec = vec![0; buffer.length() as usize];
                        buffer.copy_to(&mut vec[..]);

                        // let text = std::str::from_utf8(&vec).unwrap();
                        // alert(&format!("{:?}", text));
                        alert(&format!("{:?}", vec));
                    }) as Box<dyn FnMut()>);

                    file_reader.set_onload(Some(closure.as_ref().unchecked_ref()));

                    closure.forget();

                    file_reader.read_as_array_buffer(&file).unwrap();
                }
            }

            overlay.remove();
            cb();
        }) as Box<dyn FnMut()>);

        self.button
            .set_onclick(Some(closure.as_ref().unchecked_ref()));

        self.closure = Some(closure);

        body.append_child(&self.overlay).ok();
    }
}

impl Drop for Dialog {
    fn drop(&mut self) {
        self.button.remove();
        self.input.remove();
        self.card.remove();

        self.style.remove();
        self.overlay.remove();
    }
}

pub fn pick_file<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<PathBuf> {
    let params = params.into().unwrap_or_default();

    None
}

pub fn save_file<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<PathBuf> {
    let params = params.into().unwrap_or_default();

    None
}

pub fn pick_folder<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<PathBuf> {
    let params = params.into().unwrap_or_default();

    None
}

pub fn pick_files<'a>(params: impl Into<Option<DialogOptions<'a>>>) -> Option<Vec<PathBuf>> {
    let params = params.into().unwrap_or_default();

    None
}
