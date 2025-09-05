extern crate embed_resource;

fn main() {
    #[cfg(target_os = "windows")]
    embed_resource::compile("manifest.rc", embed_resource::NONE)
        .manifest_required()
        .unwrap();
}
