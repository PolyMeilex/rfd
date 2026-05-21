//! Web fallback backend for platforms without native file dialog support.
//!
//! This backend uses a local HTTP server (Axum) and the system's web browser to present
//! dialogs to the user. It's primarily intended for Android and other platforms
//! where native dialog APIs aren't available.

use std::time::Instant;

use crate::backend::{
    AsyncFilePickerDialogImpl, AsyncFileSaveDialogImpl, AsyncFolderPickerDialogImpl,
    AsyncMessageDialogImpl, DialogFutureType, FilePickerDialogImpl, FileSaveDialogImpl,
    FolderPickerDialogImpl, MessageDialogImpl,
};
use crate::file_dialog::FileDialog;
use crate::message_dialog::{MessageButtons, MessageDialog, MessageDialogResult};
use crate::FileHandle;
use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Router,
};
use log::{debug, warn};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot;

/// Default timeout for waiting for user response (5 minutes)
const DIALOG_TIMEOUT_SECS: u64 = 300;

// Type aliases to reduce complexity
type ResultSender<T> = Arc<Mutex<Option<mpsc::Sender<T>>>>;
type ShutdownSender = Arc<Mutex<Option<oneshot::Sender<()>>>>;

// ============================================================================
// HTML Helpers
// ============================================================================

/// Simple HTML escaping for user-provided strings
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// Build an HTML accept attribute from file filters
fn build_accept(filters: &[crate::file_dialog::Filter]) -> String {
    let mut exts: Vec<String> = Vec::new();
    for filter in filters {
        for ext in &filter.extensions {
            if !ext.is_empty() {
                exts.push(format!(".{}", ext));
            }
        }
    }
    if exts.is_empty() {
        String::new()
    } else {
        format!(r#"accept="{}""#, exts.join(","))
    }
}

/// Common HTML page wrapper with basic styling
fn html_page(title: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            margin: 0;
            padding: 20px;
            background: #f5f5f5;
            min-height: 100vh;
            display: flex;
            justify-content: center;
            align-items: center;
        }}
        .dialog {{
            background: white;
            border-radius: 8px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            padding: 24px;
            max-width: 500px;
            width: 100%;
        }}
        h1 {{
            margin: 0 0 16px 0;
            font-size: 1.5em;
            color: #333;
        }}
        p {{
            margin: 0 0 20px 0;
            color: #666;
            line-height: 1.5;
        }}
        .buttons {{
            display: flex;
            gap: 10px;
            flex-wrap: wrap;
            justify-content: flex-end;
        }}
        button, .button {{
            padding: 10px 20px;
            border: none;
            border-radius: 4px;
            font-size: 14px;
            cursor: pointer;
            text-decoration: none;
            display: inline-block;
            text-align: center;
        }}
        button[type="submit"], .button.primary {{
            background: #007bff;
            color: white;
        }}
        button[type="submit"]:hover, .button.primary:hover {{
            background: #0056b3;
        }}
        .button.secondary {{
            background: #6c757d;
            color: white;
        }}
        .button.secondary:hover {{
            background: #545b62;
        }}
        input[type="file"] {{
            margin-bottom: 20px;
            width: 100%;
        }}
        input[type="text"] {{
            width: 100%;
            padding: 10px;
            margin-bottom: 20px;
            border: 1px solid #ddd;
            border-radius: 4px;
            font-size: 14px;
        }}
        label {{
            display: block;
            margin-bottom: 8px;
            color: #333;
            font-weight: 500;
        }}
        .success {{
            text-align: center;
            color: #28a745;
        }}
    </style>
</head>
<body>
    {body}
</body>
</html>"#,
        title = html_escape(title),
        body = body
    )
}

/// Generate HTML buttons for message dialog
fn generate_buttons_html(buttons: &MessageButtons) -> String {
    match buttons {
        MessageButtons::Ok => {
            r#"<button type="submit" name="result" value="Ok">OK</button>"#.to_string()
        }
        MessageButtons::OkCancel => {
            r#"<button type="submit" name="result" value="Cancel" class="button secondary">Cancel</button>
               <button type="submit" name="result" value="Ok">OK</button>"#
                .to_string()
        }
        MessageButtons::YesNo => {
            r#"<button type="submit" name="result" value="No" class="button secondary">No</button>
               <button type="submit" name="result" value="Yes">Yes</button>"#
                .to_string()
        }
        MessageButtons::YesNoCancel => {
            r#"<button type="submit" name="result" value="Cancel" class="button secondary">Cancel</button>
               <button type="submit" name="result" value="No" class="button secondary">No</button>
               <button type="submit" name="result" value="Yes">Yes</button>"#
                .to_string()
        }
        MessageButtons::OkCustom(text) => {
            format!(
                r#"<button type="submit" name="result" value="{t}">{t}</button>"#,
                t = html_escape(text)
            )
        }
        MessageButtons::OkCancelCustom(ok, cancel) => {
            format!(
                r#"<button type="submit" name="result" value="{c}" class="button secondary">{c}</button>
                   <button type="submit" name="result" value="{o}">{o}</button>"#,
                o = html_escape(ok),
                c = html_escape(cancel)
            )
        }
        MessageButtons::YesNoCancelCustom(yes, no, cancel) => {
            format!(
                r#"<button type="submit" name="result" value="{c}" class="button secondary">{c}</button>
                   <button type="submit" name="result" value="{n}" class="button secondary">{n}</button>
                   <button type="submit" name="result" value="{y}">{y}</button>"#,
                y = html_escape(yes),
                n = html_escape(no),
                c = html_escape(cancel)
            )
        }
    }
}

/// Parse a dialog result string into MessageDialogResult
fn parse_dialog_result(result: &str) -> MessageDialogResult {
    match result {
        "Ok" | "ok" | "OK" => MessageDialogResult::Ok,
        "Cancel" | "cancel" | "CANCEL" => MessageDialogResult::Cancel,
        "Yes" | "yes" | "YES" => MessageDialogResult::Yes,
        "No" | "no" | "NO" => MessageDialogResult::No,
        custom => MessageDialogResult::Custom(custom.to_string()),
    }
}

/// Parse URL-encoded form data
fn parse_urlencoded(body: &str) -> Vec<(String, String)> {
    body.split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or("");
            Some((url_decode(key), url_decode(value)))
        })
        .collect()
}

/// Simple URL decoding
fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '%' => {
                let mut hex = String::with_capacity(2);
                for _ in 0..2 {
                    if let Some(&h) = chars.peek() {
                        if h.is_ascii_hexdigit() {
                            hex.push(chars.next().unwrap());
                        }
                    }
                }
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                    }
                }
            }
            '+' => result.push(' '),
            _ => result.push(c),
        }
    }
    result
}

// ============================================================================
// Multipart Parser
// ============================================================================

/// Parsed file from multipart form data
struct MultipartFile {
    filename: String,
    data: Vec<u8>,
}

/// Parse multipart/form-data content
fn parse_multipart(body: &[u8], boundary: &str) -> Vec<MultipartFile> {
    let mut files = Vec::new();

    // Create boundary markers
    let boundary_line = format!("--{}", boundary);
    let end_boundary_line = format!("--{}--", boundary);

    // Convert to bytes for efficient searching
    let boundary_bytes = boundary_line.as_bytes();
    let end_boundary_bytes = end_boundary_line.as_bytes();

    let mut pos = 0;
    let len = body.len();

    while pos < len {
        // Find the next boundary
        let boundary_pos = if let Some(p) = find_subsequence(&body[pos..], boundary_bytes) {
            pos + p
        } else {
            break;
        };

        // Check if this is the end boundary
        if boundary_pos + boundary_bytes.len() < len
            && &body[boundary_pos..boundary_pos + end_boundary_bytes.len()] == end_boundary_bytes
        {
            break; // End of multipart data
        }

        // Move past the boundary
        pos = boundary_pos + boundary_bytes.len();

        // Skip CRLF after boundary
        if pos + 1 < len && &body[pos..pos + 2] == b"\r\n" {
            pos += 2;
        } else if pos < len && body[pos] == b'\n' {
            pos += 1;
        }

        // Find the next boundary or end of data
        let next_boundary_pos = find_subsequence(&body[pos..], boundary_bytes)
            .map(|p| pos + p)
            .unwrap_or(len);

        let part_end = if next_boundary_pos < len {
            // Back up to before the next boundary
            next_boundary_pos
        } else {
            len
        };

        if pos >= part_end {
            break;
        }

        let part = &body[pos..part_end];

        // Parse this part
        if let Some((filename, data)) = parse_multipart_part(part) {
            if !filename.is_empty() {
                files.push(MultipartFile {
                    filename,
                    data: data.to_vec(),
                });
            }
        }

        pos = part_end;
    }

    files
}

/// Parse a single multipart part
fn parse_multipart_part(part: &[u8]) -> Option<(String, &[u8])> {
    // Find the header/body separator
    let sep_pos =
        find_subsequence(part, b"\r\n\r\n").or_else(|| find_subsequence(part, b"\n\n"))?;

    let headers = &part[..sep_pos];
    let body_start = sep_pos
        + if part.get(sep_pos..sep_pos + 4) == Some(b"\r\n\r\n") {
            4
        } else {
            2
        };

    // Remove trailing whitespace/newlines from body
    let mut body_end = part.len();
    while body_end > body_start && (part[body_end - 1] == b'\r' || part[body_end - 1] == b'\n') {
        body_end -= 1;
    }

    let body = &part[body_start..body_end];

    // Extract filename from headers
    let filename = extract_filename_from_headers(headers)?;

    Some((filename, body))
}

/// Find subsequence in byte slice
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Extract filename from Content-Disposition header bytes
fn extract_filename_from_headers(headers: &[u8]) -> Option<String> {
    let headers_str = String::from_utf8_lossy(headers);

    for line in headers_str.lines() {
        let line_trimmed = line.trim();
        if line_trimmed
            .to_lowercase()
            .starts_with("content-disposition:")
        {
            let disposition = &line_trimmed["content-disposition:".len()..].trim();

            // Parse the disposition parameters
            if let Some(params_start) = disposition.find(';') {
                let params = &disposition[params_start + 1..];

                // Look for filename parameter
                for param in params.split(';') {
                    let param = param.trim();
                    if let Some(filename_part) = param.strip_prefix("filename=") {
                        if filename_part.starts_with('"')
                            && filename_part.ends_with('"')
                            && filename_part.len() > 1
                        {
                            // Quoted filename
                            return Some(filename_part[1..filename_part.len() - 1].to_string());
                        } else if !filename_part.starts_with('"') {
                            // Unquoted filename
                            return Some(filename_part.to_string());
                        }
                    } else if let Some(encoded) = param.strip_prefix("filename*=") {
                        // RFC 6266 encoded filename
                        if let Some(quote_pos) = encoded.find("''") {
                            let charset = &encoded[..quote_pos];
                            let value = &encoded[quote_pos + 2..];
                            if charset.to_uppercase() == "UTF-8" {
                                return Some(url_decode(value));
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Extract boundary from Content-Type header
fn extract_boundary(content_type: &str) -> Option<String> {
    let content_type = content_type.trim();

    // Find the boundary parameter
    for part in content_type.split(';') {
        let part = part.trim();
        if part.to_lowercase().starts_with("boundary=") {
            let boundary = part["boundary=".len()..].trim();

            // Remove quotes if present
            if boundary.starts_with('"') && boundary.ends_with('"') && boundary.len() > 1 {
                return Some(boundary[1..boundary.len() - 1].to_string());
            } else if !boundary.is_empty() {
                return Some(boundary.to_string());
            }
        }
    }

    None
}

// ============================================================================
// Server Runner Helper
// ============================================================================

/// Run an Axum server, open browser, and wait for result
fn run_server<T: Send + 'static>(
    router: Router,
    result_rx: mpsc::Receiver<T>,
    shutdown_rx: oneshot::Receiver<()>,
) -> Option<T> {
    // Create a tokio runtime for the server
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            warn!("Web fallback: Failed to create tokio runtime: {}", e);
            return None;
        }
    };

    let result = rt.block_on(async {
        // Bind to a random available port
        let listener = match tokio::net::TcpListener::bind("127.0.0.1:0").await {
            Ok(l) => l,
            Err(e) => {
                warn!("Web fallback: Failed to bind server: {}", e);
                return None;
            }
        };

        let addr = match listener.local_addr() {
            Ok(a) => a,
            Err(e) => {
                warn!("Web fallback: Failed to get local address: {}", e);
                return None;
            }
        };

        let url = format!("http://127.0.0.1:{}/", addr.port());
        debug!("Web fallback: Server listening on {}", url);

        // Open browser
        if let Err(e) = webbrowser::open(&url) {
            warn!("Web fallback: Failed to open browser: {}", e);
            return None;
        }

        // Run server with graceful shutdown
        let server = axum::serve(listener, router).with_graceful_shutdown(async move {
            let _ = shutdown_rx.await;
        });

        tokio::spawn(async move {
            let _ = server.await;
        });

        // Wait for result with timeout
        match tokio::time::timeout(
            Duration::from_secs(DIALOG_TIMEOUT_SECS),
            tokio::task::spawn_blocking(move || result_rx.recv()),
        )
        .await
        {
            Ok(Ok(Ok(result))) => Some(result),
            Ok(Ok(Err(_))) => {
                debug!("Web fallback: Channel closed");
                None
            }
            Ok(Err(e)) => {
                warn!("Web fallback: Task error: {}", e);
                None
            }
            Err(_) => {
                debug!("Web fallback: Timeout");
                None
            }
        }
    });

    result
}

// ============================================================================
// Message Dialog Implementation
// ============================================================================

#[derive(Clone)]
struct MessageDialogState {
    title: String,
    description: String,
    buttons: MessageButtons,
    result_tx: ResultSender<MessageDialogResult>,
    shutdown_tx: ShutdownSender,
}

async fn message_dialog_page(State(state): State<MessageDialogState>) -> Html<String> {
    let buttons_html = generate_buttons_html(&state.buttons);
    let body = format!(
        r#"<div class="dialog">
            <h1>{}</h1>
            <p>{}</p>
            <form method="post" action="/submit">
                <div class="buttons">
                    {}
                </div>
            </form>
        </div>"#,
        html_escape(&state.title),
        html_escape(&state.description),
        buttons_html
    );
    Html(html_page(&state.title, &body))
}

async fn message_dialog_submit(
    State(state): State<MessageDialogState>,
    body: String,
) -> Html<String> {
    let params = parse_urlencoded(&body);
    for (key, value) in params {
        if key == "result" {
            debug!("Web fallback: Message dialog result = {}", value);
            if let Some(tx) = state.result_tx.lock().unwrap().take() {
                let _ = tx.send(parse_dialog_result(&value));
            }
            // Trigger shutdown
            if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
                let _ = tx.send(());
            }
            break;
        }
    }

    let body = r#"<div class="dialog success">
        <h1>✓</h1>
        <p>Done. You can close this tab.</p>
    </div>"#;
    Html(html_page("Done", body))
}

impl MessageDialogImpl for MessageDialog {
    fn show(self) -> MessageDialogResult {
        debug!("Web fallback: Starting message dialog");

        let (result_tx, result_rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let state = MessageDialogState {
            title: self.title.clone(),
            description: self.description.clone(),
            buttons: self.buttons.clone(),
            result_tx: Arc::new(Mutex::new(Some(result_tx))),
            shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
        };

        let router = Router::new()
            .route("/", get(message_dialog_page))
            .route("/submit", post(message_dialog_submit))
            .with_state(state);

        run_server(router, result_rx, shutdown_rx).unwrap_or(MessageDialogResult::Cancel)
    }
}

impl AsyncMessageDialogImpl for MessageDialog {
    fn show_async(self) -> DialogFutureType<MessageDialogResult> {
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let result = MessageDialogImpl::show(self);
            let _ = tx.send(result);
        });
        Box::pin(async move { rx.await.unwrap_or(MessageDialogResult::Cancel) })
    }
}

// ============================================================================
// File Picker Dialog Implementation
// ============================================================================

#[derive(Clone)]
struct FilePickerState {
    title: String,
    accept: String,
    multiple: bool,
    result_tx: ResultSender<Option<Vec<PathBuf>>>,
    shutdown_tx: ShutdownSender,
}

async fn file_picker_page(State(state): State<FilePickerState>) -> Html<String> {
    let multiple_attr = if state.multiple { "multiple" } else { "" };
    let body = format!(
        r#"<div class="dialog">
            <h1>{}</h1>
            <form method="post" action="/submit" enctype="multipart/form-data">
                <input type="file" name="files" {} {} required>
                <div class="buttons">
                    <a href="/cancel" class="button secondary">Cancel</a>
                    <button type="submit">Select</button>
                </div>
            </form>
        </div>"#,
        html_escape(&state.title),
        state.accept,
        multiple_attr
    );
    Html(html_page(&state.title, &body))
}

async fn file_picker_cancel(State(state): State<FilePickerState>) -> Html<String> {
    debug!("Web fallback: File picker cancelled");
    if let Some(tx) = state.result_tx.lock().unwrap().take() {
        let _ = tx.send(None);
    }
    if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
        let _ = tx.send(());
    }

    let body = r#"<div class="dialog success">
        <h1>Cancelled</h1>
        <p>You can close this tab.</p>
    </div>"#;
    Html(html_page("Cancelled", body))
}

async fn file_picker_submit(
    State(state): State<FilePickerState>,
    request: axum::http::Request<Body>,
) -> Response {
    debug!("Web fallback: Processing file upload");

    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let boundary = match extract_boundary(content_type) {
        Some(b) => b,
        None => {
            warn!("Web fallback: No boundary in content-type");
            if let Some(tx) = state.result_tx.lock().unwrap().take() {
                let _ = tx.send(None);
            }
            if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
                let _ = tx.send(());
            }
            return (StatusCode::BAD_REQUEST, "Missing boundary").into_response();
        }
    };

    // Read the body with increased limit for many files
    let read_start = Instant::now();
    let body_bytes = match axum::body::to_bytes(request.into_body(), 500 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            warn!("Web fallback: Failed to read body: {}", e);
            if let Some(tx) = state.result_tx.lock().unwrap().take() {
                let _ = tx.send(None);
            }
            if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
                let _ = tx.send(());
            }
            return (StatusCode::BAD_REQUEST, "Failed to read body").into_response();
        }
    };
    let read_duration = read_start.elapsed();
    debug!(
        "Web fallback: Read {} bytes in {:?}",
        body_bytes.len(),
        read_duration
    );

    let parse_start = Instant::now();
    let files = parse_multipart(&body_bytes, &boundary);
    let parse_duration = parse_start.elapsed();
    debug!(
        "Web fallback: Parsed {} files in {:?}",
        files.len(),
        parse_duration
    );

    // Create one temp directory for all files
    let temp_dir_start = Instant::now();
    let temp_dir = match tempfile::tempdir() {
        Ok(dir) => dir.keep(),
        Err(e) => {
            warn!("Web fallback: Failed to create temp dir: {}", e);
            if let Some(tx) = state.result_tx.lock().unwrap().take() {
                let _ = tx.send(None);
            }
            if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
                let _ = tx.send(());
            }
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create temp directory",
            )
                .into_response();
        }
    };
    let temp_dir_duration = temp_dir_start.elapsed();
    debug!("Web fallback: Created temp dir in {:?}", temp_dir_duration);

    let save_start = Instant::now();
    let paths: Vec<PathBuf> = files
        .into_iter()
        .filter_map(
            |f| match create_temp_file_in_dir(&temp_dir, &f.filename, &f.data) {
                Ok(path) => {
                    debug!("Web fallback: Saved file to {:?}", path);
                    Some(path)
                }
                Err(e) => {
                    warn!("Web fallback: Failed to save file {}: {}", f.filename, e);
                    None
                }
            },
        )
        .collect();
    let save_duration = save_start.elapsed();
    debug!(
        "Web fallback: Saved {} files in {:?}",
        paths.len(),
        save_duration
    );

    let result = if paths.is_empty() { None } else { Some(paths) };

    if let Some(tx) = state.result_tx.lock().unwrap().take() {
        let _ = tx.send(result);
    }
    if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
        let _ = tx.send(());
    }

    let body = r#"<div class="dialog success">
        <h1>✓</h1>
        <p>File(s) selected. You can close this tab.</p>
    </div>"#;
    Html(html_page("Done", body)).into_response()
}

fn pick_files_impl(dialog: FileDialog, multiple: bool) -> Option<Vec<PathBuf>> {
    debug!("Web fallback: Starting file picker (multiple={})", multiple);

    let (result_tx, result_rx) = mpsc::channel();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let state = FilePickerState {
        title: dialog
            .title
            .clone()
            .unwrap_or_else(|| "Select File".to_string()),
        accept: build_accept(&dialog.filters),
        multiple,
        result_tx: Arc::new(Mutex::new(Some(result_tx))),
        shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
    };

    let router = Router::new()
        .route("/", get(file_picker_page))
        .route("/cancel", get(file_picker_cancel))
        .route("/submit", post(file_picker_submit))
        .with_state(state);

    run_server(router, result_rx, shutdown_rx).flatten()
}

impl FilePickerDialogImpl for FileDialog {
    fn pick_file(self) -> Option<PathBuf> {
        pick_files_impl(self, false).and_then(|mut v| v.pop())
    }

    fn pick_files(self) -> Option<Vec<PathBuf>> {
        pick_files_impl(self, true)
    }
}

impl AsyncFilePickerDialogImpl for FileDialog {
    fn pick_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let result = FilePickerDialogImpl::pick_file(self);
            let _ = tx.send(result);
        });
        Box::pin(async move { rx.await.ok().flatten().map(FileHandle::from) })
    }

    fn pick_files_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let result = FilePickerDialogImpl::pick_files(self);
            let _ = tx.send(result);
        });
        Box::pin(async move {
            rx.await
                .ok()
                .flatten()
                .map(|v| v.into_iter().map(FileHandle::from).collect())
        })
    }
}

// ============================================================================
// Folder Picker Dialog Implementation
// ============================================================================

#[derive(Clone)]
struct FolderPickerState {
    title: String,
    multiple: bool,
    result_tx: ResultSender<Option<Vec<PathBuf>>>,
    shutdown_tx: ShutdownSender,
}

async fn folder_picker_page(State(state): State<FolderPickerState>) -> Html<String> {
    let multiple_attr = if state.multiple { "multiple" } else { "" };
    let body = format!(
        r#"<div class="dialog">
            <h1>{}</h1>
            <p>Select a folder by choosing any file within it.</p>
            <form method="post" action="/submit" enctype="multipart/form-data">
                <input type="file" name="files" webkitdirectory {} required>
                <div class="buttons">
                    <a href="/cancel" class="button secondary">Cancel</a>
                    <button type="submit">Select</button>
                </div>
            </form>
        </div>"#,
        html_escape(&state.title),
        multiple_attr
    );
    Html(html_page(&state.title, &body))
}

async fn folder_picker_cancel(State(state): State<FolderPickerState>) -> Html<String> {
    debug!("Web fallback: Folder picker cancelled");
    if let Some(tx) = state.result_tx.lock().unwrap().take() {
        let _ = tx.send(None);
    }
    if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
        let _ = tx.send(());
    }

    let body = r#"<div class="dialog success">
        <h1>Cancelled</h1>
        <p>You can close this tab.</p>
    </div>"#;
    Html(html_page("Cancelled", body))
}

async fn folder_picker_submit(
    State(state): State<FolderPickerState>,
    request: axum::http::Request<Body>,
) -> Response {
    debug!("Web fallback: Processing folder selection");

    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let boundary = match extract_boundary(content_type) {
        Some(b) => b,
        None => {
            if let Some(tx) = state.result_tx.lock().unwrap().take() {
                let _ = tx.send(None);
            }
            if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
                let _ = tx.send(());
            }
            return (StatusCode::BAD_REQUEST, "Missing boundary").into_response();
        }
    };

    let read_start = Instant::now();
    let body_bytes = match axum::body::to_bytes(request.into_body(), 500 * 1024 * 1024).await {
        Ok(b) => b,
        Err(e) => {
            warn!("Web fallback: Failed to read body: {}", e);
            if let Some(tx) = state.result_tx.lock().unwrap().take() {
                let _ = tx.send(None);
            }
            if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
                let _ = tx.send(());
            }
            return (StatusCode::BAD_REQUEST, "Failed to read body").into_response();
        }
    };
    let read_duration = read_start.elapsed();
    debug!(
        "Web fallback: Read {} bytes in {:?}",
        body_bytes.len(),
        read_duration
    );

    let parse_start = Instant::now();
    let files = parse_multipart(&body_bytes, &boundary);
    let parse_duration = parse_start.elapsed();
    debug!(
        "Web fallback: Parsed {} files in {:?}",
        files.len(),
        parse_duration
    );

    // For folder selection, we create a temp directory with the files
    let result = if files.is_empty() {
        None
    } else {
        let temp_dir_start = Instant::now();
        match tempfile::tempdir() {
            Ok(dir) => {
                let folder_path = dir.keep();
                let temp_dir_duration = temp_dir_start.elapsed();
                debug!("Web fallback: Created temp dir in {:?}", temp_dir_duration);

                let save_start = Instant::now();
                let file_count = files.len();
                // Save all files in the folder
                for file in files {
                    if let Err(e) =
                        create_temp_file_in_dir(&folder_path, &file.filename, &file.data)
                    {
                        warn!("Web fallback: Failed to write {}: {}", file.filename, e);
                    }
                }
                let save_duration = save_start.elapsed();
                debug!(
                    "Web fallback: Saved {} files in {:?}",
                    file_count, save_duration
                );
                debug!("Web fallback: Created folder at {:?}", folder_path);
                Some(vec![folder_path])
            }
            Err(e) => {
                warn!("Web fallback: Failed to create temp dir: {}", e);
                None
            }
        }
    };

    if let Some(tx) = state.result_tx.lock().unwrap().take() {
        let _ = tx.send(result);
    }
    if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
        let _ = tx.send(());
    }

    let body = r#"<div class="dialog success">
        <h1>✓</h1>
        <p>Folder selected. You can close this tab.</p>
    </div>"#;
    Html(html_page("Done", body)).into_response()
}

fn pick_folder_impl(dialog: FileDialog, multiple: bool) -> Option<Vec<PathBuf>> {
    debug!(
        "Web fallback: Starting folder picker (multiple={})",
        multiple
    );

    let (result_tx, result_rx) = mpsc::channel();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let state = FolderPickerState {
        title: dialog
            .title
            .clone()
            .unwrap_or_else(|| "Select Folder".to_string()),
        multiple,
        result_tx: Arc::new(Mutex::new(Some(result_tx))),
        shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
    };

    let router = Router::new()
        .route("/", get(folder_picker_page))
        .route("/cancel", get(folder_picker_cancel))
        .route("/submit", post(folder_picker_submit))
        .with_state(state);

    run_server(router, result_rx, shutdown_rx).flatten()
}

impl FolderPickerDialogImpl for FileDialog {
    fn pick_folder(self) -> Option<PathBuf> {
        pick_folder_impl(self, false).and_then(|mut v| v.pop())
    }

    fn pick_folders(self) -> Option<Vec<PathBuf>> {
        pick_folder_impl(self, true)
    }
}

impl AsyncFolderPickerDialogImpl for FileDialog {
    fn pick_folder_async(self) -> DialogFutureType<Option<FileHandle>> {
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let result = FolderPickerDialogImpl::pick_folder(self);
            let _ = tx.send(result);
        });
        Box::pin(async move { rx.await.ok().flatten().map(FileHandle::from) })
    }

    fn pick_folders_async(self) -> DialogFutureType<Option<Vec<FileHandle>>> {
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let result = FolderPickerDialogImpl::pick_folders(self);
            let _ = tx.send(result);
        });
        Box::pin(async move {
            rx.await
                .ok()
                .flatten()
                .map(|v| v.into_iter().map(FileHandle::from).collect())
        })
    }
}

// ============================================================================
// File Save Dialog Implementation
// ============================================================================

#[derive(Clone)]
struct FileSaveState {
    title: String,
    default_name: String,
    result_tx: ResultSender<Option<PathBuf>>,
    shutdown_tx: ShutdownSender,
}

async fn file_save_page(State(state): State<FileSaveState>) -> Html<String> {
    let body = format!(
        r#"<div class="dialog">
            <h1>{}</h1>
            <form method="post" action="/submit">
                <label for="filename">File name:</label>
                <input type="text" name="filename" id="filename" value="{}" required>
                <div class="buttons">
                    <a href="/cancel" class="button secondary">Cancel</a>
                    <button type="submit">Save</button>
                </div>
            </form>
        </div>"#,
        html_escape(&state.title),
        html_escape(&state.default_name)
    );
    Html(html_page(&state.title, &body))
}

async fn file_save_cancel(State(state): State<FileSaveState>) -> Html<String> {
    debug!("Web fallback: Save dialog cancelled");
    if let Some(tx) = state.result_tx.lock().unwrap().take() {
        let _ = tx.send(None);
    }
    if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
        let _ = tx.send(());
    }

    let body = r#"<div class="dialog success">
        <h1>Cancelled</h1>
        <p>You can close this tab.</p>
    </div>"#;
    Html(html_page("Cancelled", body))
}

async fn file_save_submit(State(state): State<FileSaveState>, body: String) -> Html<String> {
    let params = parse_urlencoded(&body);
    let mut result = None;

    for (key, value) in params {
        if key == "filename" {
            let filename = value.trim();
            if !filename.is_empty() {
                // Basic validation to prevent path traversal: disallow separators and ".."
                if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
                    warn!(
                        "Web fallback: Rejected unsafe filename from save dialog: {:?}",
                        filename
                    );
                } else {
                    // Create a path in a temp directory with the chosen filename
                    match tempfile::tempdir() {
                        Ok(dir) => {
                            let path = dir.keep().join(filename);
                            debug!("Web fallback: Save path = {:?}", path);
                            result = Some(path);
                        }
                        Err(e) => {
                            warn!("Web fallback: Failed to create temp dir: {}", e);
                        }
                    }
                }
            }
            break;
        }
    }

    if let Some(tx) = state.result_tx.lock().unwrap().take() {
        let _ = tx.send(result);
    }
    if let Some(tx) = state.shutdown_tx.lock().unwrap().take() {
        let _ = tx.send(());
    }

    let body = r#"<div class="dialog success">
        <h1>✓</h1>
        <p>Save location selected. You can close this tab.</p>
    </div>"#;
    Html(html_page("Done", body))
}

impl FileSaveDialogImpl for FileDialog {
    fn save_file(self) -> Option<PathBuf> {
        debug!("Web fallback: Starting save file dialog");

        let (result_tx, result_rx) = mpsc::channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        let state = FileSaveState {
            title: self
                .title
                .clone()
                .unwrap_or_else(|| "Save File".to_string()),
            default_name: self
                .file_name
                .clone()
                .unwrap_or_else(|| "untitled".to_string()),
            result_tx: Arc::new(Mutex::new(Some(result_tx))),
            shutdown_tx: Arc::new(Mutex::new(Some(shutdown_tx))),
        };

        let router = Router::new()
            .route("/", get(file_save_page))
            .route("/cancel", get(file_save_cancel))
            .route("/submit", post(file_save_submit))
            .with_state(state);

        run_server(router, result_rx, shutdown_rx).flatten()
    }
}

impl AsyncFileSaveDialogImpl for FileDialog {
    fn save_file_async(self) -> DialogFutureType<Option<FileHandle>> {
        let (tx, rx) = oneshot::channel();
        std::thread::spawn(move || {
            let result = FileSaveDialogImpl::save_file(self);
            let _ = tx.send(result);
        });
        Box::pin(async move { rx.await.ok().flatten().map(FileHandle::from) })
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a temporary file in the given base directory with the filename and content
fn create_temp_file_in_dir(
    base_dir: &Path,
    filename: &str,
    data: &[u8],
) -> std::io::Result<PathBuf> {
    // Sanitize the provided filename to prevent path traversal outside `base_dir`.
    let filename_path = Path::new(filename);

    // Reject absolute paths and any use of `..`, root, or platform-specific prefixes.
    if filename_path.is_absolute()
        || filename_path.components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        })
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid filename",
        ));
    }

    // Use the base directory with the sanitized relative path
    let path = base_dir.join(filename_path);
    // Create parent directories if the filename contains path separators
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = std::fs::File::create(&path)?;
    file.write_all(data)?;
    Ok(path)
}

/// Create a temporary file with the given filename and content, preserving the original filename
/// (legacy function for single files)
#[allow(dead_code)]
fn create_temp_file(filename: &str, data: &[u8]) -> std::io::Result<PathBuf> {
    // Create a temp directory to hold the file
    let temp_dir = tempfile::tempdir()?;
    let dir_path = temp_dir.keep();

    create_temp_file_in_dir(&dir_path, filename, data)
}
