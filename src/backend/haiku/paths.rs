use std::path::PathBuf;

pub(super) fn parse_paths(output: std::process::Output, save: bool) -> Option<Vec<PathBuf>> {
    // A save may print a path before B_CANCEL changes the exit status to 1.
    // Cancellation alone prints nothing and is rejected below.
    if !output.status.success() && !(save && output.status.code() == Some(1)) {
        return None;
    }
    let paths: Vec<_> = String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect();
    if paths.is_empty() {
        None
    } else {
        Some(paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;

    fn output(code: i32, stdout: &[u8]) -> std::process::Output {
        std::process::Output {
            status: std::process::ExitStatus::from_raw(if cfg!(target_os = "haiku") { code } else { code << 8 }),
            stdout: stdout.to_vec(),
            stderr: Vec::new(),
        }
    }

    #[test]
    fn save_path_survives_the_panels_cancel_exit_status() {
        assert_eq!(
            parse_paths(output(1, b"/boot/home/my document.docx\n"), true),
            Some(vec![PathBuf::from("/boot/home/my document.docx")])
        );
    }

    #[test]
    fn cancelled_panel_returns_no_paths() {
        assert!(parse_paths(output(1, b""), true).is_none());
    }

    #[test]
    fn errors_and_invalid_text_are_rejected() {
        assert!(parse_paths(output(2, b"/boot/home/document.docx\n"), true).is_none());
        assert!(parse_paths(output(1, b"/boot/home/document.docx\n"), false).is_none());
        assert!(parse_paths(output(0, &[0xff]), false).is_none());
    }

    #[test]
    fn open_preserves_multiple_paths_and_spaces() {
        assert_eq!(
            parse_paths(
                output(0, b"/boot/home/first file\n/boot/home/second\n"),
                false
            ),
            Some(vec![
                PathBuf::from("/boot/home/first file"),
                PathBuf::from("/boot/home/second")
            ])
        );
    }
}
