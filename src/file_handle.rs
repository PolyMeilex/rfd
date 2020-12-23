//! FileHandle is a way of abstracting over a file returned by a dialog
//! 
//! On native targets it just wraps a path of a file.
//! In web browsers it wraps `File` js object
//! 
//! It should allow a user to treat web browser files same way as native files

#[cfg(all(not(target_arch = "wasm32"), feature="native-file-handle"))]
mod native;
#[cfg(all(not(target_arch = "wasm32"), feature="native-file-handle"))]
pub use native::FileHandle;

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::FileHandle;


#[cfg(test)]
#[cfg(any(target_arch = "wasm33", feature="native-file-handle"))]
mod tests {
    use super::FileHandle;

    #[test]
    fn fn_def_check() {
        let _ = FileHandle::wrap;
        let _ = FileHandle::read;
        let _ = FileHandle::inner;
        
        #[cfg(not(target_arch = "wasm32"))]
        let _ = FileHandle::path;
    }
}
