use crate::file_dialog::FileDialog;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Clone, Debug)]
pub(crate) enum WasmFileHandleKind {
    Readable(web_sys::File),
    Writable(FileDialog),
    Mutable(web_sys::FileSystemFileHandle),
}

#[derive(Clone)]
pub struct FileHandle(pub(crate) WasmFileHandleKind);

impl FileHandle {
    /// Wrap a [`web_sys::File`] for reading. Use with [`FileHandle::read`]
    pub(crate) fn wrap(file: web_sys::File) -> Self {
        Self(WasmFileHandleKind::Readable(file))
    }

    /// Wrap a [`web_sys::FileSystemFileHandle`] for reading or writing. Use with [`FileHandle::write`] or [`FileHandle::read`]
    pub(crate) fn mutable(file: web_sys::FileSystemFileHandle) -> Self {
        Self(WasmFileHandleKind::Mutable(file))
    }

    /// Create a dummy `FileHandle`. Use with [`FileHandle::write`].
    pub(crate) fn writable(dialog: FileDialog) -> Self {
        FileHandle(WasmFileHandleKind::Writable(dialog))
    }

    pub fn file_name(&self) -> String {
        match &self.0 {
            WasmFileHandleKind::Readable(x) => x.name(),
            WasmFileHandleKind::Writable(x) => x.file_name.clone().unwrap_or_default(),
            WasmFileHandleKind::Mutable(x) => x.name(),
        }
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

            match &self.0 {
                WasmFileHandleKind::Readable(reader) => {
                    file_reader.read_as_array_buffer(reader).unwrap();
                }
                WasmFileHandleKind::Mutable(file) => {
                    let reader_promise = file.get_file();
                    let closure = Closure::wrap(Box::new(move |reader: JsValue| {
                        let reader: web_sys::File = reader.unchecked_into();
                        file_reader.read_as_array_buffer(&reader).unwrap();
                    }) as Box<dyn FnMut(JsValue)>);
                    let _ = reader_promise.then(&closure);
                    closure.forget();
                }
                _ => {
                    panic!("This File Handle doesn't support reading. Use `pick_file` to get a readable FileHandle");
                }
            }
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
        if let WasmFileHandleKind::Readable(reader) = &self.0 {
            reader
        } else {
            panic!("This File Handle doesn't support reading. Use `pick_file` to get a readable FileHandle");
        }
    }
}

impl std::fmt::Debug for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.file_name())
    }
}
