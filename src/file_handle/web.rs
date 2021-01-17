use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

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
