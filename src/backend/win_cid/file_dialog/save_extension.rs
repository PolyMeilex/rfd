//! Applies the file type selected in a save dialog to the extension of the returned path.

use std::path::{Path, PathBuf};

/// The path with the first extension of the file type that was selected in the save dialog, or
/// `None` when the path should stay as the dialog returned it.
///
/// - `path` is what the dialog returned.
/// - `file_name` is the name the dialog was opened with (`FileDialog::set_file_name`).
/// - `extensions` holds the extensions of the selected file type.
///
/// An extension the user typed is kept: the path only changes when it has no extension, or still
/// has the extension of `file_name` and that extension is not one of the file type's. An extension
/// the file type lists is kept whatever its case. A file type whose first extension is `*` or empty
/// accepts any name, so it never changes one.
fn with_chosen_extension(
    path: &Path,
    file_name: Option<&str>,
    extensions: &[String],
) -> Option<PathBuf> {
    let new_ext = extensions
        .first()
        .map(String::as_str)
        .filter(|ext| !ext.is_empty() && *ext != "*")?;
    path.file_name()?;

    let Some(ext) = extension(path) else {
        return Some(path.with_extension(new_ext));
    };
    if extensions
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(&ext))
    {
        return None;
    }
    let suggested = file_name.and_then(|name| extension(Path::new(name)))?;
    suggested
        .eq_ignore_ascii_case(&ext)
        .then(|| path.with_extension(new_ext))
}

/// Like `with_chosen_extension`, for the file type the dialog reports by its `index`. That is the
/// 1-based position of the type in `file_types`, the extensions of the types the dialog was given.
/// An index that names no type changes nothing.
pub(super) fn with_selected_file_type<'a>(
    path: &Path,
    file_name: Option<&str>,
    file_types: impl IntoIterator<Item = &'a [String]>,
    index: Option<u32>,
) -> Option<PathBuf> {
    let position = usize::try_from(index?.checked_sub(1)?).ok()?;
    let extensions = file_types.into_iter().nth(position)?;
    with_chosen_extension(path, file_name, extensions)
}

/// The extension of `path`, unless it is empty (`name.` has none)
fn extension(path: &Path) -> Option<String> {
    path.extension()
        .map(|ext| ext.to_string_lossy().into_owned())
        .filter(|ext| !ext.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exts(list: &[&str]) -> Vec<String> {
        list.iter().map(|ext| ext.to_string()).collect()
    }

    fn renamed(path: &str, file_name: Option<&str>, extensions: &[&str]) -> Option<PathBuf> {
        with_chosen_extension(Path::new(path), file_name, &exts(extensions))
    }

    #[test]
    fn the_suggested_extension_is_replaced() {
        assert_eq!(
            renamed("/a/Untitled.psd", Some("Untitled.psd"), &["png"]),
            Some(PathBuf::from("/a/Untitled.png"))
        );
        // The user renamed the file but kept the suggested extension
        assert_eq!(
            renamed("/a/cat.psd", Some("Untitled.psd"), &["png"]),
            Some(PathBuf::from("/a/cat.png"))
        );
        assert_eq!(
            renamed("/a/Untitled.PSD", Some("Untitled.psd"), &["png"]),
            Some(PathBuf::from("/a/Untitled.png"))
        );
        assert_eq!(
            renamed("/a/x.png.psd", Some("x.png.psd"), &["png"]),
            Some(PathBuf::from("/a/x.png.png"))
        );
    }

    #[test]
    fn a_typed_extension_is_kept() {
        assert_eq!(
            renamed("/a/Untitled.png", Some("Untitled.psd"), &["psd", "psb"]),
            None
        );
        // Without a suggested name there is nothing to tell a typed extension from
        assert_eq!(renamed("/a/Untitled.png", None, &["psd", "psb"]), None);
    }

    #[test]
    fn an_extension_the_file_type_lists_is_kept() {
        assert_eq!(
            renamed("/a/Untitled.psd", Some("Untitled.psd"), &["psd", "psb"]),
            None
        );
        assert_eq!(
            renamed("/a/Untitled.psb", Some("Untitled.psd"), &["psd", "psb"]),
            None
        );
        assert_eq!(
            renamed("/a/Untitled.PSB", Some("Untitled.psd"), &["psd", "psb"]),
            None
        );
    }

    #[test]
    fn a_missing_extension_is_added() {
        assert_eq!(
            renamed("/a/Untitled", Some("Untitled.psd"), &["psd", "psb"]),
            Some(PathBuf::from("/a/Untitled.psd"))
        );
        assert_eq!(
            renamed("/a/Untitled", None, &["psd", "psb"]),
            Some(PathBuf::from("/a/Untitled.psd"))
        );
        // A dot with nothing after it is not an extension
        assert_eq!(
            renamed("/a/Untitled.", None, &["png"]),
            Some(PathBuf::from("/a/Untitled.png"))
        );
    }

    #[test]
    fn a_file_type_without_an_extension_changes_nothing() {
        for any in [&["*"][..], &[""][..], &[][..]] {
            assert_eq!(renamed("/a/Untitled.psd", Some("Untitled.psd"), any), None);
            assert_eq!(renamed("/a/Untitled", None, any), None);
        }
        // Only the first extension decides
        assert_eq!(renamed("/a/Untitled", None, &["*", "png"]), None);
    }

    #[test]
    fn the_file_type_is_found_by_its_one_based_index() {
        let types = [
            exts(&["psd", "psb"]),
            exts(&["png"]),
            exts(&["jpg", "jpeg"]),
        ];
        let selected = |index| {
            with_selected_file_type(
                Path::new("/a/Untitled.psd"),
                Some("Untitled.psd"),
                types.iter().map(Vec::as_slice),
                index,
            )
        };
        // The type the name already has is kept
        assert_eq!(selected(Some(1)), None);
        assert_eq!(selected(Some(2)), Some(PathBuf::from("/a/Untitled.png")));
        assert_eq!(selected(Some(3)), Some(PathBuf::from("/a/Untitled.jpg")));
    }

    #[test]
    fn an_index_that_names_no_file_type_changes_nothing() {
        let types = [exts(&["png"]), exts(&["jpg"])];
        for index in [None, Some(0), Some(3), Some(u32::MAX)] {
            assert_eq!(
                with_selected_file_type(
                    Path::new("/a/Untitled.psd"),
                    Some("Untitled.psd"),
                    types.iter().map(Vec::as_slice),
                    index,
                ),
                None
            );
        }
        // A dialog with no file types has none to select
        assert_eq!(
            with_selected_file_type(
                Path::new("/a/Untitled.psd"),
                Some("Untitled.psd"),
                std::iter::empty(),
                Some(1),
            ),
            None
        );
    }

    #[test]
    fn a_path_without_a_file_name_stays() {
        assert_eq!(renamed("", None, &["png"]), None);
        assert_eq!(renamed("/", None, &["png"]), None);
    }
}
