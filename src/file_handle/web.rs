use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{Blob, HtmlAnchorElement};

pub struct FileHandle(web_sys::File);

impl FileHandle {
    pub fn wrap(file: web_sys::File) -> Self {
        Self(file)
    }

    pub fn file_name(&self) -> String {
        self.0.name()
    }

    // Path is not supported in browsers.
    // Use read() instead.
    // pub fn path(&self) -> &Path {
    //     // compile_error!();
    //     unimplemented!("Path is not supported in browsers");
    // }

    pub async fn read(&self) -> Vec<u8> {
        let promise = js_sys::Promise::new(&mut move |res, _rej| {
            let file_reader = web_sys::FileReader::new().unwrap();

            let fr = file_reader.clone();
            let closure = Closure::wrap(Box::new(move || {
                res.call1(&JsValue::undefined(), &fr.result().unwrap())
                    .unwrap();
            }) as Box<dyn FnMut()>);

            file_reader.set_onload(Some(closure.as_ref().unchecked_ref()));

            closure.forget();

            file_reader.read_as_array_buffer(&self.0).unwrap();
        });

        let future = wasm_bindgen_futures::JsFuture::from(promise);

        let res = future.await.unwrap();

        let buffer: js_sys::Uint8Array = js_sys::Uint8Array::new(&res);
        let mut vec = vec![0; buffer.length() as usize];
        buffer.copy_to(&mut vec[..]);

        vec
    }

    pub async fn write(data: Box<[u8]>) {
        {
            let window = web_sys::window().expect("Window not found");
            let document = window.document().expect("Document not found");
            let body = document.body().expect("document should have a body");

            let overlay = document.create_element("div").unwrap();
            overlay.set_id("rfd-overlay");

            let card = {
                let card = document.create_element("div").unwrap();
                card.set_id("rfd-card");
                overlay.append_child(&card).unwrap();

                card
            };

            let output = {
                let output_el = document.create_element("a").unwrap();
                let output: HtmlAnchorElement = wasm_bindgen::JsCast::dyn_into(output_el).unwrap();

                output.set_id("rfd-output");
                output.set_inner_text("click here to download your file");

                card.append_child(&output).unwrap();
                output
            };

            let button = {
                let btn_el = document.create_element("button").unwrap();
                let btn: web_sys::HtmlButtonElement =
                    wasm_bindgen::JsCast::dyn_into(btn_el).unwrap();

                btn.set_id("rfd-button");
                btn.set_inner_text("Create file");

                card.append_child(&btn).unwrap();
                btn
            };

            let style = document.create_element("style").unwrap();
            style.set_inner_html(include_str!("../backend/wasm/style.css"));
            overlay.append_child(&style).unwrap();

            body.append_child(&overlay).unwrap();

            let promise = js_sys::Promise::new(&mut |res, _rej| {
                // Clones to keep closure as FnMut
                let output2 = output.clone();
                let data = data.clone();

                let set_download_link = move |in_array: &[u8], name: &str| {
                    // See <https://stackoverflow.com/questions/69556755/web-sysurlcreate-object-url-with-blobblob-not-formatting-binary-data-co>
                    let array = js_sys::Array::new();
                    let uint8arr = js_sys::Uint8Array::new(
                        // Safety: No wasm allocations happen between creating the view and consuming it in the array.push
                        &unsafe { js_sys::Uint8Array::view(&in_array) }.into(),
                    );
                    array.push(&uint8arr.buffer());
                    let blob = Blob::new_with_u8_array_sequence_and_options(
                        &array,
                        web_sys::BlobPropertyBag::new().type_("application/octet-stream"),
                    )
                    .unwrap();
                    let download_url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();

                    output2.set_href(&download_url);
                    output2.set_download(&name);
                };

                let set_download_link = Closure::wrap(Box::new(move || {
                    set_download_link(&*data, "file.txt");
                }) as Box<dyn FnMut()>);

                let end_promise = Closure::wrap(Box::new(move || {
                    res.call1(&JsValue::undefined(), &JsValue::from(true))
                        .unwrap(); // End promise
                }) as Box<dyn FnMut()>);

                button.set_onclick(Some(set_download_link.as_ref().unchecked_ref()));
                set_download_link.forget();

                // Resolve the promise once the user clicks the download link.
                output.set_onclick(Some(end_promise.as_ref().unchecked_ref()));
                end_promise.forget();

                body.append_child(&overlay).ok();
            });
            let future = wasm_bindgen_futures::JsFuture::from(promise);
            future.await.unwrap();

            // Drop impl
            style.remove();
            button.remove();
            output.remove();
            card.remove();
            overlay.remove();
        }
    }

    #[cfg(feature = "file-handle-inner")]
    pub fn inner(&self) -> &web_sys::File {
        &self.0
    }
}

impl std::fmt::Debug for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_name())
    }
}
