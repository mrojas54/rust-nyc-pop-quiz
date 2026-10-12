//! Keyboard scheduling form backed by the existing Python scheduler.
use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver, TryRecvError},
};

pub(crate) struct Schedule {
    id: String,
    bank: PathBuf,
    fields: [String; 3],
    field: usize,
    confirm: bool,
    finished: bool,
    message: String,
    job: Option<Receiver<String>>,
    scroll: u16,
    max_scroll: u16,
}

impl Schedule {
    pub(crate) fn new(id: String, bank: PathBuf) -> Self {
        Self {
            id,
            bank,
            fields: Default::default(),
            field: 0,
            confirm: false,
            finished: false,
            message: String::new(),
            job: None,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::PageDown => self.scroll = self.scroll.saturating_add(10).min(self.max_scroll),
            KeyCode::PageUp => self.scroll = self.scroll.saturating_sub(10),
            KeyCode::Down | KeyCode::Up if self.finished || self.confirm => {
                self.scroll = if key == KeyCode::Down {
                    self.scroll.saturating_add(1).min(self.max_scroll)
                } else {
                    self.scroll.saturating_sub(1)
                };
            }
            KeyCode::Down | KeyCode::Tab
                if !self.confirm && !self.finished && self.job.is_none() =>
            {
                self.field = (self.field + 1) % 3;
                self.scroll = 0;
            }
            KeyCode::Up | KeyCode::BackTab
                if !self.confirm && !self.finished && self.job.is_none() =>
            {
                self.field = (self.field + 2) % 3;
                self.scroll = 0;
            }
            KeyCode::Char(c) => return self.handle(c),
            KeyCode::Enter => return self.handle('\n'),
            KeyCode::Esc => return self.handle('\u{1b}'),
            KeyCode::Backspace => return self.handle('\u{8}'),
            _ => {}
        }
        true
    }

    /// False closes the form. No external action happens before an explicit y.
    pub(crate) fn handle(&mut self, key: char) -> bool {
        if self.job.is_some() {
            return true;
        }
        if self.finished {
            match key {
                '\u{1b}' => return false,
                'j' => self.scroll = self.scroll.saturating_add(1).min(self.max_scroll),
                'k' => self.scroll = self.scroll.saturating_sub(1),
                _ => {}
            }
            return true;
        }
        if self.confirm {
            match key {
                '\u{1b}' => self.confirm = false,
                'y' => self.start(),
                _ => {}
            }
            return true;
        }
        match key {
            '\u{1b}' => return false,
            '\t' => self.field = (self.field + 1) % 3,
            '\u{8}' => {
                self.fields[self.field].pop();
            }
            '\n' => {
                self.message = self.validate().err().unwrap_or_default();
                self.confirm = self.message.is_empty();
            }
            c if !c.is_control() && self.fields[self.field].len() < 2048 => {
                self.fields[self.field].push(c)
            }
            _ => {}
        }
        true
    }

    fn validate(&self) -> Result<(), String> {
        if !valid_date(&self.fields[0]) {
            return Err("Enter a real date in YYYY-MM-DD form.".into());
        }
        if self.fields[1].trim().is_empty() {
            return Err("Enter the room URL.".into());
        }
        if !Path::new(&self.fields[2]).is_absolute() {
            return Err("Use an absolute fallback directory outside the repository.".into());
        }
        let output = resolved_path(Path::new(&self.fields[2]))
            .map_err(|_| "Cannot resolve the fallback directory.".to_string())?;
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .map_err(|_| "Cannot resolve the repository directory.".to_string())?;
        if output.starts_with(repo) {
            return Err("Choose a fallback directory outside the repository.".into());
        }
        Ok(())
    }

    fn command(&self) -> Command {
        let pipeline = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../pipeline");
        let python = std::env::var_os("POPQUIZ_PYTHON")
            .map(PathBuf::from)
            .unwrap_or_else(|| pipeline.join(".venv/bin/python"));
        let mut cmd = Command::new(python);
        cmd.current_dir(&pipeline)
            .env("PYTHONPATH", pipeline.join("src"))
            .args([
                "-m",
                "popquiz.schedule",
                "schedule",
                &self.id,
                "--date",
                &self.fields[0],
                "--room",
                &self.fields[1],
                "--out",
                &self.fields[2],
                "--bank",
            ])
            .arg(&self.bank)
            .stdin(Stdio::null());
        cmd
    }

    fn start(&mut self) {
        let mut cmd = self.command();
        let (tx, rx) = mpsc::channel();
        self.job = Some(rx);
        self.message = "Scheduling… waiting for the room. Do not resubmit.\nClosing the terminal does not undo a submitted request.".into();
        std::thread::spawn(move || {
            let result = match cmd.output() {
                Ok(out) => {
                    let heading = result_heading(out.status.success());
                    format!("{heading}\n{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr))
                },
                Err(_) => "Could not start the scheduler. Set up pipeline/.venv with Python 3.12, or set POPQUIZ_PYTHON to its executable. Nothing was sent.".into(),
            };
            let _ = tx.send(safe_report(result));
        });
    }

    pub(crate) fn poll(&mut self) {
        let Some(job) = &self.job else {
            return;
        };
        let result = match job.try_recv() {
            Ok(message) => message,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => "Scheduler stopped without a result. The remote outcome is unknown; check the room before retrying.".into(),
        };
        self.message = result;
        self.job = None;
        self.finished = true;
    }

    pub(crate) fn text(&self) -> String {
        let mut text = format!("Schedule {}\n\n", self.id);
        if self.finished || self.job.is_some() {
            text.push_str(&self.message);
            if self.finished {
                text.push_str("\n\nEscape Back   j/k Scroll result");
            }
            return text;
        }
        let labels = ["Date (YYYY-MM-DD)", "Room URL", "Fallback directory"];
        for (i, label) in labels.iter().enumerate() {
            text.push_str(&format!(
                "{} {label}: {}\n",
                if self.field == i { ">" } else { " " },
                self.fields[i]
            ));
        }
        if self.confirm {
            text.push_str(&format!("\nBank: {}\n\nSend this question to the room and write its fallback and host sheet.\nAn existing scheduled record or fallback file may be replaced.\nThe scheduler rechecks acceptance, affirmation, verification, and prior use.\nScheduling does not open a room or mark the question used.\n\ny Schedule   Escape Edit", self.bank.display()));
        } else {
            text.push_str("\nTab Next field   Backspace Delete   Enter Review   Escape Cancel\n\nThe admin token comes from POPQUIZ_ADMIN_TOKEN.\nThe fallback contains the answer; choose a private directory.\n");
            text.push_str(&self.message);
        }
        text
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Schedule question / PgUp PgDn Scroll ")
            .border_style(Style::default().fg(super::AMBER));
        let inner = block.inner(area);
        let text = self.text();
        let lines: Vec<_> = text
            .lines()
            .flat_map(|line| textwrap::wrap(line, usize::from(inner.width.max(1))))
            .map(|line| Line::raw(line.into_owned()))
            .collect();
        self.max_scroll = lines
            .len()
            .saturating_sub(usize::from(inner.height))
            .min(usize::from(u16::MAX)) as u16;
        self.scroll = self.scroll.min(self.max_scroll);
        frame.render_widget(block, area);
        frame.render_widget(Paragraph::new(lines).scroll((self.scroll, 0)), inner);
    }
}

fn result_heading(success: bool) -> &'static str {
    if success {
        "Scheduled"
    } else {
        "Scheduler reported a problem"
    }
}

// Resolve existing symlink prefixes as well as new directories and parent components.
fn resolved_path(path: &Path) -> std::io::Result<PathBuf> {
    use std::path::Component;
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            other => {
                result.push(other.as_os_str());
                match result.canonicalize() {
                    Ok(canonical) => result = canonical,
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e),
                }
            }
        }
    }
    Ok(result)
}

fn safe_report(mut text: String) -> String {
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

fn valid_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(i, b)| i == 4 || i == 7 || b.is_ascii_digit())
    {
        return false;
    }
    let year: u32 = text[..4].parse().unwrap_or(0);
    let month: usize = text[5..7].parse().unwrap_or(0);
    let day: u32 = text[8..].parse().unwrap_or(0);
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    year > 0 && (1..=12).contains(&month) && day > 0 && day <= days[month.saturating_sub(1).min(11)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_paths_are_refused_before_confirmation() {
        let mut form = Schedule::new("q3".into(), PathBuf::from("/tmp/bank"));
        form.fields = [
            "2026-10-15".into(),
            "https://example.test".into(),
            format!("{}/../../new-fallback", env!("CARGO_MANIFEST_DIR")),
        ];
        form.handle('\n');
        assert!(!form.confirm);
        assert!(form.message.contains("outside the repository"));
        assert_eq!(
            resolved_path(Path::new("/tmp/new-dir/../fallback")).unwrap(),
            Path::new("/tmp").canonicalize().unwrap().join("fallback")
        );
    }

    #[test]
    fn result_scroll_clamps_to_rendered_height_and_resize() {
        use ratatui::{backend::TestBackend, Terminal};
        let mut form = Schedule::new("q3".into(), PathBuf::from("/tmp/bank"));
        form.finished = true;
        form.message = "many wrapped words to test scrolling. ".repeat(100);
        let mut terminal = Terminal::new(TestBackend::new(40, 14)).unwrap();
        terminal.draw(|f| form.draw(f, f.area())).unwrap();
        for _ in 0..1000 {
            form.handle('j');
        }
        assert_eq!(form.scroll, form.max_scroll);
        assert!(form.scroll > 0);
        form.message = "short".into();
        terminal.draw(|f| form.draw(f, f.area())).unwrap();
        assert_eq!(form.scroll, 0);
    }

    #[test]
    fn unsuccessful_exit_does_not_claim_the_room_was_unchanged() {
        assert_eq!(result_heading(false), "Scheduler reported a problem");
    }

    #[test]
    fn dates_reject_impossible_days_and_accept_leap_years() {
        for date in [
            "2026-02-29",
            "2026-04-31",
            "0000-01-01",
            "2026-1-01",
            "ééééé",
        ] {
            assert!(!valid_date(date));
        }
        assert!(valid_date("2028-02-29"));
    }

    #[test]
    fn review_is_not_submission_and_escape_preserves_fields() {
        let mut form = Schedule::new("q3".into(), PathBuf::from("/tmp/bank"));
        for c in "2026-10-15\thttps://example.test\t/tmp/private fallback\n".chars() {
            form.handle(c);
        }
        assert!(form.confirm);
        assert!(form.job.is_none());
        form.handle('\n');
        assert!(form.job.is_none());
        form.handle('\u{1b}');
        assert!(!form.confirm);
        assert_eq!(form.fields[2], "/tmp/private fallback");
        assert!(form
            .command()
            .get_args()
            .any(|arg| arg == "/tmp/private fallback"));
    }

    #[test]
    fn scheduler_failure_is_reported_without_retrying() {
        let mut form = Schedule::new("q3".into(), PathBuf::from("/tmp/bank"));
        let (tx, rx) = mpsc::channel();
        form.job = Some(rx);
        tx.send("q3 is not affirmed".into()).unwrap();
        form.poll();
        assert!(form.finished);
        assert!(form.text().contains("not affirmed"));
        form.handle('y');
        assert!(form.job.is_none());
    }
}
