//! Running the Python pipeline as a subprocess: the scheduler and the review CLI.
//! The TUI never writes a bank record itself; these modules do.
use std::{
    io,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::mpsc::{self, Receiver},
};

/// `python -m <module>` from `pipeline/`, with `POPQUIZ_PYTHON` or `pipeline/.venv/bin/python`.
pub(crate) fn python(module: &str) -> Command {
    let pipeline = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pipeline");
    let python = std::env::var_os("POPQUIZ_PYTHON")
        .map(PathBuf::from)
        .unwrap_or_else(|| pipeline.join(".venv/bin/python"));
    let mut cmd = Command::new(python);
    cmd.current_dir(&pipeline)
        .env("PYTHONPATH", pipeline.join("src"))
        .args(["-m", module])
        .stdin(Stdio::null());
    cmd
}

/// Run `cmd` on a background thread; the receiver gets `finish`'s view of the result.
pub(crate) fn spawn<T: Send + 'static>(
    mut cmd: Command,
    finish: impl FnOnce(io::Result<Output>) -> T + Send + 'static,
) -> Receiver<T> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(finish(cmd.output()));
    });
    rx
}

/// Subprocess output made safe to show: the admin token redacted, control characters dropped.
pub(crate) fn safe_report(mut text: String) -> String {
    if let Ok(token) = std::env::var("POPQUIZ_ADMIN_TOKEN") {
        if !token.trim().is_empty() {
            text = text.replace(token.trim(), "[redacted]");
        }
    }
    text.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t'))
        .take(16000)
        .collect()
}
