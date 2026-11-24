//! Keeps the extension of a save panel's name in step with the selected format.

/// The name to show in a save panel once the user has selected another format, or `None` when the
/// name should stay as it is.
///
/// - `name` is what the name field holds now.
/// - `previous` holds the extensions of the format that was selected until now.
/// - `suggested` is the name the panel was opened with (`FileDialog::set_file_name`).
/// - `selected` holds the extensions of the format that has just been selected.
///
/// The name gets the first extension of the selected format when it has none, or when it still has
/// the extension of the previous format or of the suggested name, and that extension is not one of
/// the selected format's own. An extension the user typed is kept, and so is any extension the
/// selected format lists, whatever its case. A format whose first extension is `*` or empty accepts
/// any name, so it never changes one.
pub(super) fn name_for_format(
    name: &str,
    previous: &[String],
    suggested: Option<&str>,
    selected: &[String],
) -> Option<String> {
    let new_ext = selected
        .first()
        .map(String::as_str)
        .filter(|ext| !ext.is_empty() && *ext != "*")?;
    if name.trim().is_empty() {
        return None;
    }

    let Some((stem, ext)) = split_extension(name) else {
        // No extension, or only dots at the end
        let stem = name.trim_end_matches('.');
        return (!stem.is_empty()).then(|| format!("{stem}.{new_ext}"));
    };
    if has_extension(selected, ext) {
        return None;
    }
    let suggested_ext = suggested
        .and_then(split_extension)
        .map(|(_, suggested_ext)| suggested_ext);
    let was_ours = has_extension(previous, ext)
        || suggested_ext.is_some_and(|suggested_ext| suggested_ext.eq_ignore_ascii_case(ext));
    was_ours.then(|| format!("{stem}.{new_ext}"))
}

/// The part of `name` before its extension, and the extension. A leading dot does not start an
/// extension (`.plan` has none), and neither does a dot at the end.
fn split_extension(name: &str) -> Option<(&str, &str)> {
    name.rsplit_once('.')
        .filter(|(stem, ext)| !stem.is_empty() && !ext.is_empty())
}

fn has_extension(extensions: &[String], ext: &str) -> bool {
    extensions
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(ext))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exts(list: &[&str]) -> Vec<String> {
        list.iter().map(|ext| ext.to_string()).collect()
    }

    fn renamed(
        name: &str,
        previous: &[&str],
        suggested: Option<&str>,
        selected: &[&str],
    ) -> Option<String> {
        name_for_format(name, &exts(previous), suggested, &exts(selected))
    }

    #[test]
    fn the_previous_formats_extension_is_replaced() {
        assert_eq!(
            renamed("Untitled.psd", &["psd"], None, &["png"]),
            Some("Untitled.png".to_string())
        );
        // A name the user changed keeps its stem
        assert_eq!(
            renamed("Holiday.psd", &["psd"], None, &["tiff"]),
            Some("Holiday.tiff".to_string())
        );
        // The case of the old extension does not matter
        assert_eq!(
            renamed("Holiday.PixMap", &["pixmap"], None, &["tiff"]),
            Some("Holiday.tiff".to_string())
        );
    }

    #[test]
    fn any_extension_of_the_previous_format_is_replaced() {
        assert_eq!(
            renamed("Holiday.jpeg", &["jpg", "jpeg"], None, &["png"]),
            Some("Holiday.png".to_string())
        );
    }

    #[test]
    fn only_the_last_extension_is_replaced() {
        assert_eq!(
            renamed("archive.tar.psd", &["psd"], None, &["png"]),
            Some("archive.tar.png".to_string())
        );
    }

    #[test]
    fn the_extension_of_the_suggested_name_is_replaced() {
        // The suggested extension is not one the first format lists
        assert_eq!(
            renamed("cat.psd", &["png"], Some("Untitled.psd"), &["jpg"]),
            Some("cat.jpg".to_string())
        );
        assert_eq!(
            renamed("cat.PSD", &["png"], Some("Untitled.psd"), &["jpg"]),
            Some("cat.jpg".to_string())
        );
    }

    #[test]
    fn a_typed_extension_is_kept() {
        assert_eq!(
            renamed("notes.txt", &["psd"], Some("Untitled.psd"), &["png"]),
            None
        );
        // Without a suggested name there is nothing to tell a typed extension from
        assert_eq!(renamed("notes.txt", &["psd"], None, &["png"]), None);
    }

    #[test]
    fn an_extension_the_selected_format_lists_is_kept() {
        assert_eq!(
            renamed(
                "Holiday.tiff",
                &["psd"],
                Some("Holiday.tiff"),
                &["tiff", "tif"]
            ),
            None
        );
        assert_eq!(
            renamed("Holiday.tif", &["psd"], None, &["tiff", "tif"]),
            None
        );
        assert_eq!(
            renamed("Holiday.TIF", &["psd"], None, &["tiff", "tif"]),
            None
        );
    }

    #[test]
    fn a_missing_extension_is_added() {
        assert_eq!(
            renamed("Untitled", &["psd"], Some("Untitled.psd"), &["tiff", "tif"]),
            Some("Untitled.tiff".to_string())
        );
        assert_eq!(
            renamed("Untitled", &["psd"], None, &["png"]),
            Some("Untitled.png".to_string())
        );
        // A hidden file's leading dot does not start an extension
        assert_eq!(
            renamed(".notes", &["psd"], None, &["png"]),
            Some(".notes.png".to_string())
        );
        // A dot with nothing after it is not an extension either
        assert_eq!(
            renamed("Untitled.", &["psd"], None, &["png"]),
            Some("Untitled.png".to_string())
        );
    }

    #[test]
    fn a_format_without_an_extension_changes_nothing() {
        for any in [&["*"][..], &[""][..], &[][..]] {
            assert_eq!(
                renamed("Untitled.psd", &["psd"], Some("Untitled.psd"), any),
                None
            );
            assert_eq!(renamed("Untitled", &["psd"], None, any), None);
        }
        // Only the first extension decides
        assert_eq!(renamed("Untitled", &["psd"], None, &["*", "png"]), None);
    }

    #[test]
    fn an_empty_name_stays_empty() {
        assert_eq!(renamed("", &["psd"], None, &["png"]), None);
        assert_eq!(renamed("  ", &["psd"], None, &["png"]), None);
        assert_eq!(renamed("..", &["psd"], None, &["png"]), None);
    }

    #[test]
    fn a_name_follows_a_series_of_formats() {
        let formats = [
            exts(&["psd", "psb"]),
            exts(&["png"]),
            exts(&["tiff", "tif"]),
            exts(&["psd", "psb"]),
        ];
        let mut name = "Untitled.psd".to_string();
        for pair in formats.windows(2) {
            if let Some(next) = name_for_format(&name, &pair[0], Some("Untitled.psd"), &pair[1]) {
                name = next;
            }
        }
        assert_eq!(name, "Untitled.psd");

        // A format that accepts any name leaves the extension, and the next format replaces it
        let mut name = "Untitled.png".to_string();
        assert_eq!(
            name_for_format(&name, &exts(&["png"]), None, &exts(&["*"])),
            None
        );
        name = name_for_format(&name, &exts(&["*"]), Some("Untitled.png"), &exts(&["tiff"]))
            .unwrap_or(name);
        assert_eq!(name, "Untitled.tiff");
    }
}
