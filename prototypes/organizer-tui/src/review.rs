//! Review decisions on one question, through `python -m popquiz.review`.
//!
//! The screen shows what `review show` reports and runs the CLI for each decision.
//! The CLI is the one writer: this module writes no bank record, never `verified`,
//! and never invents an organizer name (rulings 1-3).
use crate::pipeline;
use crossterm::event::KeyCode;
use ratatui::{
    layout::Rect,
    style::Style,
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use serde_json::Value;
use std::{
    path::PathBuf,
    process::Command,
    sync::mpsc::{Receiver, TryRecvError},
};

/// Where the organizer's name comes from. There is no default.
const ORGANIZER_ENV: &str = "POPQUIZ_ORGANIZER";

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Decision {
    Accept(Vec<String>),
    Reject(String),
    Difficulty(u8),
    Affirm,
    Edit(PathBuf),
}

#[derive(Debug, PartialEq)]
enum Stage {
    Notice,
    Screen,
    Reason,
    Difficulty,
    Confirm(Decision),
    Editing(PathBuf),
    Result,
}

enum Job {
    Show,
    Export(PathBuf),
    Decide(Decision),
}

struct Outcome {
    ok: bool,
    stdout: String,
    stderr: String,
}

pub(crate) struct Review {
    id: String,
    bank: PathBuf,
    organizer: Option<String>,
    editor: Option<String>,
    stage: Stage,
    report: Option<Value>,
    message: String,
    input: String,
    job: Option<(Job, Receiver<Option<Outcome>>)>,
    editor_request: Option<(String, PathBuf)>,
    /// Set once a decision ran, so the preview reloads its records on close.
    pub(crate) changed: bool,
    scroll: u16,
    max_scroll: u16,
}

fn from_env(names: &[&str]) -> Option<String> {
    names
        .iter()
        .filter_map(std::env::var_os)
        .map(|v| v.to_string_lossy().trim().to_string())
        .find(|v| !v.is_empty())
}

impl Review {
    pub(crate) fn new(id: String, bank: PathBuf) -> Self {
        Self::with(
            id,
            bank,
            from_env(&[ORGANIZER_ENV]),
            from_env(&["VISUAL", "EDITOR"]),
        )
    }

    fn with(id: String, bank: PathBuf, organizer: Option<String>, editor: Option<String>) -> Self {
        Self {
            id,
            bank,
            organizer,
            editor,
            stage: Stage::Notice,
            report: None,
            message: String::new(),
            input: String::new(),
            job: None,
            editor_request: None,
            changed: false,
            scroll: 0,
            max_scroll: 0,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char(c) => self.handle(c),
            KeyCode::Enter => self.handle('\n'),
            KeyCode::Esc => self.handle('\u{1b}'),
            KeyCode::Backspace => self.handle('\u{8}'),
            KeyCode::Down if self.stage != Stage::Reason => self.handle('j'),
            KeyCode::Up if self.stage != Stage::Reason => self.handle('k'),
            KeyCode::PageDown => {
                self.scroll = self.scroll.saturating_add(10).min(self.max_scroll);
                true
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(10);
                true
            }
            _ => true,
        }
    }

    /// False closes the screen. Nothing is written before an explicit y.
    pub(crate) fn handle(&mut self, key: char) -> bool {
        if self.job.is_some() || matches!(self.stage, Stage::Editing(_)) {
            return true;
        }
        match std::mem::replace(&mut self.stage, Stage::Screen) {
            Stage::Notice => match key {
                '\n' => self.start(Job::Show),
                '\u{1b}' => return false,
                _ => self.stage = Stage::Notice,
            },
            Stage::Screen => return self.screen_key(key),
            Stage::Reason => match key {
                '\u{1b}' => {}
                '\n' if self.input.trim().is_empty() => {
                    self.message = "A rejection needs a reason.".into();
                    self.stage = Stage::Reason;
                }
                '\n' => self.stage = Stage::Confirm(Decision::Reject(self.input.trim().into())),
                '\u{8}' => {
                    self.input.pop();
                    self.stage = Stage::Reason;
                }
                c => {
                    if !c.is_control() && self.input.len() < 2048 {
                        self.input.push(c);
                    }
                    self.stage = Stage::Reason;
                }
            },
            Stage::Difficulty => match key {
                '1'..='5' => {
                    self.stage = Stage::Confirm(Decision::Difficulty(key as u8 - b'0'));
                }
                '\u{1b}' => {}
                _ => self.stage = Stage::Difficulty,
            },
            Stage::Confirm(decision) => match key {
                'y' => self.start(Job::Decide(decision)),
                '\u{1b}' => {
                    if let Decision::Edit(path) = decision {
                        let _ = std::fs::remove_file(path);
                        self.message = "Edit discarded; nothing was saved.".into();
                    }
                }
                _ => self.stage = Stage::Confirm(decision),
            },
            Stage::Result => match key {
                '\n' | '\u{1b}' => self.start(Job::Show),
                _ => self.stage = Stage::Result,
            },
            Stage::Editing(path) => self.stage = Stage::Editing(path),
        }
        true
    }

    fn screen_key(&mut self, key: char) -> bool {
        self.stage = Stage::Screen;
        match key {
            '\u{1b}' => return false,
            'j' => self.scroll = self.scroll.saturating_add(1).min(self.max_scroll),
            'k' => self.scroll = self.scroll.saturating_sub(1),
            _ if self.report.is_none() => {}
            'a' => self.stage = Stage::Confirm(Decision::Accept(self.unchecked_quotes())),
            'x' => {
                self.input.clear();
                self.stage = Stage::Reason;
            }
            'd' => self.stage = Stage::Difficulty,
            'A' => self.stage = Stage::Confirm(Decision::Affirm),
            'e' if self.editor.is_none() => {
                self.message = "Set VISUAL or EDITOR to edit a question.".into();
            }
            'e' => {
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos())
                    .unwrap_or_default();
                let path = std::env::temp_dir().join(format!(
                    "popquiz-review-{}-{}-{nanos}.json",
                    self.id,
                    std::process::id()
                ));
                self.start(Job::Export(path));
            }
            _ => {}
        }
        true
    }

    /// Quoted spans that are not a printed line and are not checked yet (AC-73).
    fn unchecked_quotes(&self) -> Vec<String> {
        self.field("quotes")
            .as_array()
            .into_iter()
            .flatten()
            .filter(|q| q["matches"] != true && q["checked"] != true)
            .filter_map(|q| q["span"].as_str().map(String::from))
            .collect()
    }

    fn field(&self, key: &str) -> &Value {
        self.report.as_ref().map_or(&Value::Null, |r| &r[key])
    }

    fn command(&self, job: &Job) -> Command {
        let mut cmd = pipeline::python("popquiz.review");
        // The name shown on the confirm screen is the only name that can be used.
        cmd.env_remove(ORGANIZER_ENV);
        match job {
            Job::Show => {
                cmd.arg("show");
            }
            Job::Export(path) => {
                cmd.args(["export", "--out"]).arg(path);
            }
            Job::Decide(Decision::Accept(quotes)) => {
                cmd.arg("accept");
                for quote in quotes {
                    cmd.arg(format!("--checked-quote={quote}"));
                }
            }
            Job::Decide(Decision::Reject(reason)) => {
                cmd.arg("reject").arg(format!("--reason={reason}"));
            }
            Job::Decide(Decision::Difficulty(_)) => {
                cmd.arg("difficulty");
            }
            Job::Decide(Decision::Affirm) => {
                cmd.arg("affirm");
                if let Some(name) = &self.organizer {
                    cmd.arg(format!("--by={name}"));
                }
            }
            Job::Decide(Decision::Edit(path)) => {
                cmd.args(["edit", "--from"]).arg(path);
            }
        }
        // Options go first: argparse reads everything after `--` as positional.
        cmd.arg("--bank").arg(&self.bank).args(["--", &self.id]);
        if let Job::Decide(Decision::Difficulty(level)) = job {
            cmd.arg(level.to_string());
        }
        cmd
    }

    fn start(&mut self, job: Job) {
        let cmd = self.command(&job);
        self.message.clear();
        self.stage = Stage::Screen;
        self.scroll = 0;
        let rx = pipeline::spawn(cmd, |result| {
            result.ok().map(|out| Outcome {
                ok: out.status.success(),
                stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
            })
        });
        self.job = Some((job, rx));
    }

    pub(crate) fn poll(&mut self) {
        let Some((_, rx)) = &self.job else {
            return;
        };
        let outcome = match rx.try_recv() {
            Ok(outcome) => outcome,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => None,
        };
        let (job, _) = self.job.take().expect("checked above");
        let Some(out) = outcome else {
            self.message = "Could not start the review CLI. Set up pipeline/.venv, or set POPQUIZ_PYTHON. Nothing was written.".into();
            return;
        };
        let said = pipeline::safe_report(format!("{}{}", out.stdout, out.stderr))
            .trim()
            .to_string();
        match job {
            Job::Show => match serde_json::from_str(&out.stdout) {
                Ok(report) if out.ok => {
                    self.report = Some(report);
                }
                _ => self.message = format!("Could not load the review.\n{said}"),
            },
            Job::Export(path) if out.ok => {
                self.editor_request = self.editor.clone().map(|editor| (editor, path.clone()));
                self.stage = Stage::Editing(path);
            }
            Job::Export(path) => {
                let _ = std::fs::remove_file(path);
                self.message = format!("Could not prepare the edit.\n{said}");
            }
            Job::Decide(decision) => {
                // Sticky: a refusal after an earlier decision still leaves that one to reload.
                self.changed |= out.ok;
                let mut message = format!(
                    "{}\n{said}",
                    if out.ok { "Recorded." } else { "Not recorded." }
                );
                if let Decision::Edit(path) = decision {
                    if out.ok {
                        let _ = std::fs::remove_file(path);
                    } else {
                        message.push_str(&format!("\nYour edit is kept in {}.", path.display()));
                    }
                }
                self.message = message;
                self.stage = Stage::Result;
            }
        }
    }

    /// The editor the main loop should open, with the file `review export` wrote.
    pub(crate) fn take_editor_request(&mut self) -> Option<(String, PathBuf)> {
        self.editor_request.take()
    }

    pub(crate) fn editor_finished(&mut self, ok: bool) {
        let Stage::Editing(path) = std::mem::replace(&mut self.stage, Stage::Screen) else {
            return;
        };
        if ok {
            self.stage = Stage::Confirm(Decision::Edit(path));
        } else {
            let _ = std::fs::remove_file(path);
            self.message = "The editor did not exit cleanly; nothing was saved.".into();
        }
    }

    pub(crate) fn text(&self) -> String {
        let mut t = format!("Review {}\n\n", self.id);
        if self.stage == Stage::Notice {
            t.push_str("Review shows the answer. You will have seen it; the room has not.\n\nEnter Continue   Esc Back");
            return t;
        }
        if let Some((job, _)) = &self.job {
            t.push_str(match job {
                Job::Show => "Loading the review…",
                Job::Export(_) => "Preparing the edit…",
                Job::Decide(Decision::Edit(_)) => "Saving the edit… a source or choice change re-verifies with rustc and Miri and may take minutes.",
                Job::Decide(_) => "Recording…",
            });
            return t;
        }
        match &self.stage {
            Stage::Result => {
                t.push_str(&self.message);
                t.push_str("\n\nEnter Back to the review");
                return t;
            }
            Stage::Editing(path) => {
                t.push_str(&format!("Editing {} in your editor…", path.display()));
                return t;
            }
            _ => {}
        }
        if self.report.is_some() {
            self.report_text(&mut t);
        }
        t.push('\n');
        match &self.stage {
            Stage::Reason => t.push_str(&format!(
                "Reject: why? {}\nEnter Continue   Esc Cancel\n",
                self.input
            )),
            Stage::Difficulty => t.push_str("Difficulty judged: press 1 to 5   Esc Cancel\n"),
            Stage::Confirm(decision) => {
                t.push_str(&match decision {
                    Decision::Accept(quotes) if quotes.is_empty() => format!("Accept {}?", self.id),
                    Decision::Accept(quotes) => format!(
                        "Accept {}, recording that you checked these quotes are not claims about the output: {}?",
                        self.id,
                        quotes.iter().map(|q| format!("\"{q}\"")).collect::<Vec<_>>().join(", ")
                    ),
                    Decision::Reject(reason) => format!("Reject {}: {reason}?", self.id),
                    Decision::Difficulty(level) => format!("Record difficulty {level} as judged?"),
                    Decision::Affirm => format!(
                        "Affirm {}? Organizer: {}",
                        self.id,
                        self.organizer.as_deref().unwrap_or("not set (set POPQUIZ_ORGANIZER)")
                    ),
                    Decision::Edit(_) => "Save this edit? It clears any affirmation. A source or choice change re-verifies with rustc and Miri.".into(),
                });
                t.push_str("\ny Yes   Esc Cancel\n");
            }
            _ => t.push_str(
                "a Accept   x Reject   d Difficulty   e Edit   A Affirm   j/k Scroll   Esc Back\n",
            ),
        }
        if !self.message.is_empty() {
            t.push('\n');
            t.push_str(&self.message);
        }
        t
    }

    fn report_text(&self, t: &mut String) {
        let r = |key: &str| self.field(key);
        let s = |v: &Value| v.as_str().unwrap_or("").to_string();
        let warnings = |t: &mut String, field: &str| {
            for w in r("warnings").as_array().into_iter().flatten() {
                if w["field"] == field {
                    t.push_str(&format!(
                        "  warning (does not block): {} \"{}\"\n",
                        s(&w["group"]),
                        s(&w["matched"])
                    ));
                }
            }
        };
        t.push_str(&format!("Topic: {}\n", s(r("topic"))));
        let status = s(r("status"));
        t.push_str(&format!(
            "Status: {}{}\n",
            if status.is_empty() {
                "not reviewed"
            } else {
                &status
            },
            r("reason")
                .as_str()
                .map(|why| format!(" ({why})"))
                .unwrap_or_default()
        ));
        let by = format!("{} at {}", s(r("affirmed_by")), s(r("affirmed_at")));
        t.push_str(&format!(
            "Affirmation: {}\n",
            match r("affirmation").as_str() {
                Some("bound") => format!("affirmed by {by}"),
                Some("stale") => format!("NOT AFFIRMED: the text changed after {by} affirmed it"),
                Some("unbound") => format!(
                    "NOT AFFIRMED: {by} recorded no digest of what was affirmed; affirm it again"
                ),
                _ => "not affirmed".into(),
            }
        ));
        t.push_str(&format!(
            "Difficulty: requested {}, judged {}\n",
            r("difficulty_requested"),
            r("difficulty_judged")
                .as_u64()
                .map_or("not yet".into(), |d| d.to_string())
        ));
        t.push_str(&format!(
            "Near duplicate of: {}\n",
            r("near_duplicate_of").as_str().unwrap_or("none recorded")
        ));
        if let Some(ruling) = r("blocked_on").as_str() {
            t.push_str(&format!(
                "WAITING on ruling {ruling}: it cannot be affirmed yet.\n"
            ));
        }
        t.push_str("\nSource\n");
        for (i, line) in s(r("source")).lines().enumerate() {
            t.push_str(&format!("{:3}  {line}\n", i + 1));
        }
        t.push_str("\nChoices (bank order, not meetup letters)\n");
        for (i, (letter, o)) in ('A'..='E')
            .zip(r("options").as_array().into_iter().flatten())
            .enumerate()
        {
            let correct = if o["correct"] == true {
                "   [correct, machine record]"
            } else {
                ""
            };
            t.push_str(&format!("{letter}  {}{correct}\n", s(&o["text"])));
            if let Some(why) = o["why_tempting"].as_str() {
                t.push_str(&format!("   Why tempting: {why}\n"));
            }
            warnings(t, &format!("options[{i}].why_tempting"));
        }
        if let Some(error) = r("answer_error").as_str() {
            t.push_str(&format!("No correct answer derives: {error}\n"));
        }
        t.push_str(&format!("\nHint (human-written): {}\n", s(r("hint"))));
        warnings(t, "hint");
        t.push_str(&format!("What happened: {}\n", s(&r("explains")["what"])));
        warnings(t, "explains.what");
        t.push_str(&format!(
            "What to remember: {}\n",
            s(&r("explains")["takeaway"])
        ));
        warnings(t, "explains.takeaway");
        t.push_str(&format!("Trace: {} steps\n", r("trace_steps")));
        t.push_str("\nHow we know / machine record\n");
        let receipt = r("receipt").as_array().cloned().unwrap_or_default();
        if receipt.is_empty() {
            t.push_str("No complete receipt recorded\n");
        }
        for line in &receipt {
            t.push_str(&format!("{}\n", s(line)));
        }
        if r("legacy_record") == true {
            t.push_str("Legacy record: Miri ran outside the verifier.\n");
        }
        t.push_str("Miri evidence covers executed paths only.\n");
        let quotes = r("quotes").as_array().cloned().unwrap_or_default();
        if !quotes.is_empty() {
            t.push_str("\nQuoted output\n");
        }
        for q in &quotes {
            let state = if q["matches"] == true {
                "matches a printed line"
            } else if q["checked"] == true {
                "not a printed line, checked"
            } else {
                "not a printed line, BLOCKS accept until checked"
            };
            t.push_str(&format!("\"{}\": {state}\n", s(&q["span"])));
        }
        for (label, key) in [("Accept", "accept_blockers"), ("Affirm", "affirm_blockers")] {
            let blockers: Vec<_> = r(key).as_array().into_iter().flatten().map(s).collect();
            if blockers.is_empty() {
                t.push_str(&format!("\n{label}: READY\n"));
            } else {
                t.push_str(&format!("\n{label}: BLOCKED\n"));
                for b in blockers {
                    t.push_str(&format!("  {b}\n"));
                }
            }
        }
    }

    pub(crate) fn draw(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Review question / PgUp PgDn Scroll ")
            .border_style(Style::default().fg(super::AMBER));
        let inner = block.inner(area);
        let text = self.text();
        let lines: Vec<_> = text
            .lines()
            .map(|line| {
                line.chars()
                    .map(|c| if c.is_control() { ' ' } else { c })
                    .collect::<String>()
            })
            .flat_map(|line| {
                textwrap::wrap(&line, usize::from(inner.width.max(1)))
                    .into_iter()
                    .map(|l| Line::raw(l.into_owned()))
                    .collect::<Vec<_>>()
            })
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn review(organizer: Option<&str>) -> Review {
        Review::with(
            "q4".into(),
            PathBuf::from("/tmp/bank"),
            organizer.map(String::from),
            None,
        )
    }

    fn args(cmd: &Command) -> Vec<String> {
        cmd.get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    fn removes_organizer(cmd: &Command) -> bool {
        cmd.get_envs()
            .any(|(k, v)| k == ORGANIZER_ENV && v.is_none())
    }

    #[test]
    fn affirm_passes_only_the_name_shown_and_never_a_default() {
        let unset = review(None);
        let cmd = unset.command(&Job::Decide(Decision::Affirm));
        assert!(!args(&cmd).iter().any(|a| a.starts_with("--by")));
        assert!(removes_organizer(&cmd));
        let set = review(Some("Organizer Name"));
        let cmd = set.command(&Job::Decide(Decision::Affirm));
        assert!(args(&cmd).contains(&"--by=Organizer Name".to_string()));
        assert!(removes_organizer(&cmd));
    }

    #[test]
    fn notice_comes_before_any_answer_and_no_job_starts_without_enter() {
        let mut r = review(None);
        assert!(r.text().contains("the room has not"));
        r.handle('a');
        r.handle('y');
        assert!(r.job.is_none());
        assert!(!r.handle('\u{1b}'));
    }

    #[test]
    fn every_decision_waits_for_y_and_escape_cancels() {
        let mut r = review(None);
        r.stage = Stage::Screen;
        r.report = Some(json!({"quotes": [
            {"span": "seen", "matches": true, "checked": false},
            {"span": "unseen", "matches": false, "checked": false},
            {"span": "done", "matches": false, "checked": true}
        ]}));
        r.handle('a');
        assert_eq!(
            r.stage,
            Stage::Confirm(Decision::Accept(vec!["unseen".into()]))
        );
        assert!(r.job.is_none());
        r.handle('\u{1b}');
        for c in "x  \n".chars() {
            r.handle(c);
        }
        assert_eq!(r.stage, Stage::Reason);
        assert!(r.text().contains("needs a reason"));
        for c in "Too close to q3\n".chars() {
            r.handle(c);
        }
        assert_eq!(
            r.stage,
            Stage::Confirm(Decision::Reject("Too close to q3".into()))
        );
        r.handle('\u{1b}');
        r.handle('d');
        r.handle('9');
        r.handle('4');
        assert_eq!(r.stage, Stage::Confirm(Decision::Difficulty(4)));
        assert!(r.job.is_none());
    }

    #[test]
    fn checked_quotes_and_reasons_reach_the_cli_as_arguments() {
        // Joined with `=`, so a value that looks like a flag ("-x", "--foo") is
        // still read as the value.
        let r = review(None);
        let cmd = r.command(&Job::Decide(Decision::Accept(vec![
            "-3 -1".into(),
            "-x".into(),
        ])));
        assert_eq!(
            args(&cmd)[2..],
            [
                "accept",
                "--checked-quote=-3 -1",
                "--checked-quote=-x",
                "--bank",
                "/tmp/bank",
                "--",
                "q4"
            ]
        );
        let cmd = r.command(&Job::Decide(Decision::Reject("--foo".into())));
        assert_eq!(args(&cmd)[2..4], ["reject", "--reason=--foo"]);
    }

    #[test]
    fn the_id_comes_after_double_dash_with_every_option_before_it() {
        let named = review(Some("Name"));
        let end = ["--bank", "/tmp/bank", "--", "q4"];
        for (job, start) in [
            (Job::Show, vec!["show"]),
            (
                Job::Export(PathBuf::from("/tmp/out.json")),
                vec!["export", "--out", "/tmp/out.json"],
            ),
            (Job::Decide(Decision::Accept(vec![])), vec!["accept"]),
            (
                Job::Decide(Decision::Reject("why".into())),
                vec!["reject", "--reason=why"],
            ),
            (Job::Decide(Decision::Affirm), vec!["affirm", "--by=Name"]),
            (
                Job::Decide(Decision::Edit(PathBuf::from("/tmp/in.json"))),
                vec!["edit", "--from", "/tmp/in.json"],
            ),
        ] {
            assert_eq!(
                args(&named.command(&job))[2..],
                [start, end.to_vec()].concat()
            );
        }
        // The level is the one positional after the id.
        let cmd = named.command(&Job::Decide(Decision::Difficulty(4)));
        assert_eq!(
            args(&cmd)[2..],
            ["difficulty", "--bank", "/tmp/bank", "--", "q4", "4"]
        );
    }

    #[test]
    fn an_id_that_looks_like_a_flag_is_still_the_id() {
        let r = Review::with("-x".into(), PathBuf::from("/tmp/bank"), None, None);
        assert_eq!(
            args(&r.command(&Job::Show))[2..],
            ["show", "--bank", "/tmp/bank", "--", "-x"]
        );
    }

    fn finish_decision(r: &mut Review, ok: bool) {
        let (tx, rx) = std::sync::mpsc::channel();
        tx.send(Some(Outcome {
            ok,
            stdout: String::new(),
            stderr: String::new(),
        }))
        .unwrap();
        r.job = Some((Job::Decide(Decision::Affirm), rx));
        r.poll();
    }

    #[test]
    fn a_refused_decision_does_not_ask_for_a_reload() {
        let mut r = review(None);
        finish_decision(&mut r, false);
        assert!(r.text().contains("Not recorded."));
        assert!(!r.changed);
    }

    #[test]
    fn a_refusal_does_not_undo_the_reload_an_earlier_decision_asked_for() {
        let mut r = review(None);
        finish_decision(&mut r, true);
        assert!(r.text().contains("Recorded."));
        assert!(r.changed);
        finish_decision(&mut r, false);
        assert!(r.text().contains("Not recorded."));
        assert!(r.changed);
    }

    #[test]
    fn edit_needs_an_editor_and_has_no_default() {
        let mut r = review(None);
        r.stage = Stage::Screen;
        r.report = Some(json!({}));
        r.handle('e');
        assert!(r.job.is_none());
        assert!(r.text().contains("Set VISUAL or EDITOR"));
    }

    #[test]
    fn states_are_words_and_warnings_sit_beside_their_field() {
        let mut r = review(None);
        r.stage = Stage::Screen;
        r.report = Some(json!({
            "status": "accepted", "affirmation": "unbound",
            "blocked_on": "F-15",
            "options": [{"text": "1", "correct": true, "why_tempting": null},
                        {"text": "2", "correct": false, "why_tempting": "Looks right."}],
            "hint": "Look closely.",
            "explains": {"what": "w", "takeaway": "literally t"},
            "warnings": [{"field": "explains.takeaway", "group": "filler", "matched": "literally"}],
            "accept_blockers": [], "affirm_blockers": ["It does not compile."]
        }));
        let text = r.text();
        assert!(text.contains("Affirmation: NOT AFFIRMED: "));
        assert!(text.contains("recorded no digest of what was affirmed; affirm it again"));
        assert!(text.contains("WAITING on ruling F-15"));
        assert!(text.contains("A  1   [correct, machine record]"));
        assert!(text.contains(
            "What to remember: literally t\n  warning (does not block): filler \"literally\""
        ));
        assert!(text.contains("Accept: READY"));
        assert!(text.contains("Affirm: BLOCKED\n  It does not compile."));
    }
}
