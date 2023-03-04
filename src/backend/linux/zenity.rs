use futures_util::AsyncReadExt;
use std::{
    error::Error,
    fmt::Display,
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

use super::child_stdout::ChildStdout;
use crate::{
    file_dialog::Filter,
    message_dialog::{MessageButtons, MessageLevel},
    FileDialog,
};

#[derive(Debug)]
pub enum ZenityError {
    Io(std::io::Error),
    StdOutNotFound,
}

impl Error for ZenityError {}

impl Display for ZenityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZenityError::Io(io) => write!(f, "{io}"),
            ZenityError::StdOutNotFound => write!(f, "Stdout not found"),
        }
    }
}

impl From<std::io::Error> for ZenityError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub type ZenityResult<T> = Result<T, ZenityError>;

fn command() -> Command {
    Command::new("zenity")
}

fn add_filters(command: &mut Command, filters: &[Filter]) {
    for f in filters.iter() {
        command.arg("--file-filter");

        let extensions: Vec<_> = f
            .extensions
            .iter()
            .map(|ext| format!("*.{}", ext))
            .collect();

        command.arg(format!("{} | {}", f.name, extensions.join(" ")));
    }
}

fn add_filename(command: &mut Command, file_name: &Option<String>) {
    if let Some(name) = file_name.as_ref() {
        command.arg("--filename");
        command.arg(name);
    }
}

async fn run(mut command: Command) -> ZenityResult<Option<String>> {
    let mut process = command.stdout(Stdio::piped()).spawn()?;

    let stdout = process.stdout.take().ok_or(ZenityError::StdOutNotFound)?;
    let mut stdout = ChildStdout::new(stdout)?;

    let mut buffer = String::new();
    stdout.read_to_string(&mut buffer).await?;

    let status = loop {
        if let Some(status) = process.try_wait()? {
            break status;
        }

        async_io::Timer::after(Duration::from_millis(1)).await;
    };

    Ok(if status.success() { Some(buffer) } else { None })
}

#[allow(unused)]
pub async fn pick_file(dialog: &FileDialog) -> ZenityResult<Option<PathBuf>> {
    let mut command = command();
    command.arg("--file-selection");

    add_filters(&mut command, &dialog.filters);
    add_filename(&mut command, &dialog.file_name);

    run(command).await.map(|res| {
        res.map(|buffer| {
            let trimed = buffer.trim();
            trimed.into()
        })
    })
}

#[allow(unused)]
pub async fn pick_files(dialog: &FileDialog) -> ZenityResult<Vec<PathBuf>> {
    let mut command = command();
    command.args(["--file-selection", "--multiple"]);

    add_filters(&mut command, &dialog.filters);
    add_filename(&mut command, &dialog.file_name);

    run(command).await.map(|res| {
        res.map(|buffer| {
            let list = buffer.trim().split('|').map(PathBuf::from).collect();
            list
        })
        .unwrap_or(Vec::new())
    })
}

#[allow(unused)]
pub async fn pick_folder(dialog: &FileDialog) -> ZenityResult<Option<PathBuf>> {
    let mut command = command();
    command.args(["--file-selection", "--directory"]);

    add_filters(&mut command, &dialog.filters);
    add_filename(&mut command, &dialog.file_name);

    run(command).await.map(|res| {
        res.map(|buffer| {
            let trimed = buffer.trim();
            trimed.into()
        })
    })
}

#[allow(unused)]
pub async fn save_file(dialog: &FileDialog) -> ZenityResult<Option<PathBuf>> {
    let mut command = command();
    command.args(["--file-selection", "--save", "--confirm-overwrite"]);

    add_filters(&mut command, &dialog.filters);
    add_filename(&mut command, &dialog.file_name);

    run(command).await.map(|res| {
        res.map(|buffer| {
            let trimed = buffer.trim();
            trimed.into()
        })
    })
}

pub async fn message(
    level: &MessageLevel,
    btns: &MessageButtons,
    title: &str,
    description: &str,
) -> ZenityResult<bool> {
    let cmd = match level {
        MessageLevel::Info => "--info",
        MessageLevel::Warning => "--warning",
        MessageLevel::Error => "--error",
    };

    let ok_label = match btns {
        MessageButtons::Ok => None,
        MessageButtons::OkCustom(ok) => Some(ok),
        _ => None,
    };

    let mut command = command();
    command.args([cmd, "--title", title, "--text", description]);

    if let Some(ok) = ok_label {
        command.args(["--ok-label", ok]);
    }

    run(command).await.map(|res| res.is_some())
}

pub async fn question(btns: &MessageButtons, title: &str, description: &str) -> ZenityResult<bool> {
    let labels = match btns {
        MessageButtons::OkCancel => Some(("Ok", "Cancel")),
        MessageButtons::YesNo => None,
        MessageButtons::OkCancelCustom(ok, cancel) => Some((ok.as_str(), cancel.as_str())),
        _ => None,
    };

    let mut command = command();
    command.args(["--question", "--title", title, "--text", description]);

    if let Some((ok, cancel)) = labels {
        command.args(["--ok-label", ok]);
        command.args(["--cancel-label", cancel]);
    }

    run(command).await.map(|res| res.is_some())
}

#[cfg(test)]
mod tests {
    use crate::FileDialog;

    #[test]
    #[ignore]
    fn message() {
        async_io::block_on(super::message(
            &crate::message_dialog::MessageLevel::Info,
            &crate::message_dialog::MessageButtons::Ok,
            "hi",
            "me",
        ))
        .unwrap();
        async_io::block_on(super::message(
            &crate::message_dialog::MessageLevel::Warning,
            &crate::message_dialog::MessageButtons::Ok,
            "hi",
            "me",
        ))
        .unwrap();
        async_io::block_on(super::message(
            &crate::message_dialog::MessageLevel::Error,
            &crate::message_dialog::MessageButtons::Ok,
            "hi",
            "me",
        ))
        .unwrap();
    }

    #[test]
    #[ignore]
    fn question() {
        async_io::block_on(super::question(
            &crate::message_dialog::MessageButtons::OkCancel,
            "hi",
            "me",
        ))
        .unwrap();
        async_io::block_on(super::question(
            &crate::message_dialog::MessageButtons::YesNo,
            "hi",
            "me",
        ))
        .unwrap();
    }

    #[test]
    #[ignore]
    fn pick_file() {
        let path = async_io::block_on(super::pick_file(&FileDialog::default())).unwrap();
        dbg!(path);
    }

    #[test]
    #[ignore]
    fn pick_files() {
        let path = async_io::block_on(super::pick_files(&FileDialog::default())).unwrap();
        dbg!(path);
    }

    #[test]
    #[ignore]
    fn pick_folder() {
        let path = async_io::block_on(super::pick_folder(&FileDialog::default())).unwrap();
        dbg!(path);
    }

    #[test]
    #[ignore]
    fn save_file() {
        let path = async_io::block_on(super::save_file(&FileDialog::default())).unwrap();
        dbg!(path);
    }
}
