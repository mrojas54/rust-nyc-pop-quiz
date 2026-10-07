//! Read-only question and authored teaching-trace preview.
use ratatui::{
    backend::TestBackend,
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};
use serde::Deserialize;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Deserialize)]
struct Choice {
    text: String,
    why_tempting: Option<String>,
}
#[derive(Default, Deserialize)]
struct Trace {
    #[serde(default)]
    steps: Vec<Step>,
}
#[derive(Deserialize)]
struct Step {
    lines: Vec<usize>,
    focus: [usize; 2],
    note: String,
    #[serde(default)]
    values: Vec<Delta>,
    #[serde(default)]
    pivot: bool,
}
#[derive(Deserialize)]
struct Delta {
    name: String,
    was: String,
    now: String,
}
#[derive(Deserialize)]
struct Explains {
    what: String,
    takeaway: String,
}
#[derive(Deserialize)]
struct Candidate {
    id: String,
    source: String,
    topic: String,
    options: Vec<Choice>,
    #[serde(default)]
    trace: Trace,
    explains: Explains,
    #[serde(skip)]
    correct: Option<usize>,
    #[serde(skip)]
    receipt: Vec<String>,
    #[serde(skip)]
    legacy: bool,
}

impl Candidate {
    fn parse(raw: &str) -> Result<Self> {
        // Share answer derivation with the live room, never author an answer.
        let record = room::answers::Record::from_json(raw)?;
        let mut candidate: Self = serde_json::from_str(raw)?;
        if candidate.options.len() != 5 {
            return Err("A candidate must contain five options".into());
        }
        candidate.correct = record.correct_index()?;
        let json: serde_json::Value = serde_json::from_str(raw)?;
        if let Some(value) = json.get("verified").filter(|v| !v.is_null()) {
            let verified = room::answers::verified_from_json(value)?;
            candidate.receipt = room::answers::receipt_lines(&verified).unwrap_or_default();
            candidate.legacy = value
                .get("legacy")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
        }
        Ok(candidate)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Question,
    Trace,
    Reveal,
}

pub struct App {
    candidates: Vec<Candidate>,
    index: usize,
    mode: Mode,
    step: usize,
    scroll: usize,
    max_scroll: usize,
}

pub fn default_bank() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bank")
}

impl App {
    pub fn load(bank: &Path, question: Option<&str>) -> Result<Self> {
        let mut paths = fs::read_dir(bank.join("questions"))?
            .map(|entry| entry.map(|e| e.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        paths.retain(|path| path.extension().is_some_and(|ext| ext == "json"));
        paths.sort();
        let candidates = paths
            .iter()
            .map(|path| {
                Candidate::parse(&fs::read_to_string(path)?)
                    .map_err(|error| format!("{}: {error}", path.display()).into())
            })
            .collect::<Result<Vec<_>>>()?;
        if candidates.is_empty() {
            return Err("No candidates found in this bank".into());
        }
        let index = match question {
            Some(id) => candidates
                .iter()
                .position(|q| q.id == id)
                .ok_or_else(|| format!("Candidate {id:?} was not found"))?,
            None => 0,
        };
        Ok(Self {
            candidates,
            index,
            mode: Mode::Question,
            step: 0,
            scroll: 0,
            max_scroll: 0,
        })
    }

    /// Returns false only for an explicit quit action.
    pub fn handle(&mut self, key: char) -> bool {
        let total = self.candidates[self.index].trace.steps.len();
        let before = (self.index, self.mode, self.step);
        match key {
            'q' | '\u{1b}' => return false,
            '1' => self.mode = Mode::Question,
            't' | '\t' => {
                self.mode = if self.mode == Mode::Trace {
                    Mode::Question
                } else {
                    Mode::Trace
                };
                self.step = self.step.min(total.saturating_sub(2));
            }
            'r' => {
                self.mode = Mode::Reveal;
                self.step = total.saturating_sub(1);
            }
            '>' if self.mode != Mode::Question => {
                let reserve = if self.mode == Mode::Trace { 2 } else { 1 };
                self.step = (self.step + 1).min(total.saturating_sub(reserve));
            }
            '<' if self.mode != Mode::Question => self.step = self.step.saturating_sub(1),
            '[' | ']' => {
                let count = self.candidates.len();
                self.index = (self.index + if key == ']' { 1 } else { count - 1 }) % count;
                self.mode = Mode::Question;
                self.step = 0;
            }
            'j' => self.scroll = (self.scroll + 1).min(self.max_scroll),
            'k' => self.scroll = self.scroll.saturating_sub(1),
            'J' => self.scroll = (self.scroll + 10).min(self.max_scroll),
            'K' => self.scroll = self.scroll.saturating_sub(10),
            _ => {}
        }
        if before != (self.index, self.mode, self.step) {
            self.scroll = 0;
        }
        true
    }

    fn rows(&self, width: usize) -> Vec<Line<'static>> {
        let q = &self.candidates[self.index];
        let mut rows = Vec::new();
        let normal = Style::default();
        let heading = Style::default().add_modifier(Modifier::BOLD);
        let muted = Style::default().add_modifier(Modifier::DIM);
        let active = Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD);
        let mut add = |text: &str, style: Style| {
            let safe: String = text
                .chars()
                .map(|c| if c.is_control() && c != '\n' { ' ' } else { c })
                .collect();
            for paragraph in safe.split('\n') {
                for wrapped in textwrap::wrap(paragraph, width.max(1)) {
                    rows.push(Line::styled(wrapped.into_owned(), style));
                }
            }
        };
        let step = if self.mode == Mode::Reveal
            || (self.mode == Mode::Trace && q.trace.steps.len() >= 2)
        {
            q.trace.steps.get(self.step)
        } else {
            None
        };
        add(
            if self.mode == Mode::Question {
                "What happens when this program runs?"
            } else {
                "Let's walk it."
            },
            heading,
        );
        add("", normal);
        for (i, source) in q.source.lines().enumerate() {
            let number = i + 1;
            let highlighted = step.is_some_and(|s| s.lines.contains(&number));
            let style = if highlighted {
                active
            } else if step.is_some_and(|s| number < s.focus[0] || number > s.focus[1]) {
                muted
            } else {
                normal
            };
            add(
                &format!(
                    "{} {number:2}  {}",
                    if highlighted { ">" } else { " " },
                    source.replace('\t', "    ")
                ),
                style,
            );
        }
        add("", normal);
        match self.mode {
            Mode::Question => {
                add("Choices / bank order, not meetup letters", muted);
                for (letter, choice) in ('A'..='E').zip(&q.options) {
                    add(&format!("{letter}  {}", choice.text), normal);
                }
                add("", normal);
                add("Press t to walk through the teaching trace.", heading);
            }
            _ => {
                if let Some(s) = step {
                    add(
                        &format!(
                            "Step {} of {}{}",
                            self.step + 1,
                            q.trace.steps.len(),
                            if s.pivot { " / Pause here." } else { "" }
                        ),
                        heading,
                    );
                    add("Host narration / authored teaching walkthrough", muted);
                    add(&s.note, normal);
                    if !s.values.is_empty() {
                        add("", normal);
                        add(
                            "Illustrated values / authored, not debugger captures",
                            muted,
                        );
                        for value in &s.values {
                            // Match the live wall: stdout belongs to the reveal.
                            if self.mode == Mode::Trace && value.name == "stdout" {
                                continue;
                            }
                            add(
                                &format!("{}: {} -> {}", value.name, value.was, value.now),
                                normal,
                            );
                        }
                    }
                    if self.mode == Mode::Trace
                        && self.step == q.trace.steps.len().saturating_sub(2)
                    {
                        add("", normal);
                        add(
                            "Teaching steps complete. Press r to preview reveal.",
                            heading,
                        );
                    }
                } else {
                    add(
                        "No teaching steps available before reveal. At least two steps are needed.",
                        normal,
                    );
                }
            }
        }
        if self.mode == Mode::Reveal {
            add("", normal);
            let answer = q
                .correct
                .and_then(|i| q.options.get(i))
                .map(|o| o.text.as_str())
                .unwrap_or("Unavailable from the recorded evidence");
            add(&format!("Correct answer / {answer}"), heading);
            add(
                "Existing verification record; no new verification was run.",
                muted,
            );
            add("", normal);
            add("What happened", heading);
            add(&q.explains.what, normal);
            add("", normal);
            add("What to remember", heading);
            add(&q.explains.takeaway, normal);
            add("", normal);
            add("How we know / recorded evidence", heading);
            if q.receipt.is_empty() {
                add("No complete receipt recorded", normal);
            }
            for line in &q.receipt {
                add(line, normal);
            }
            if q.legacy {
                add("Legacy record: Miri ran outside the verifier.", normal);
            }
            add("Miri evidence covers executed paths only.", muted);
            add("", normal);
            add("Why the other choices are tempting", heading);
            for option in &q.options {
                if let Some(why) = &option.why_tempting {
                    add(&option.text, heading);
                    add(why, normal);
                    add("", normal);
                }
            }
        }
        rows
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        if area.width < 40 || area.height < 14 {
            frame.render_widget(Paragraph::new("Resize to 40 x 14 or larger. q Quit"), area);
            return;
        }
        let [header, tabs, content, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .areas(area);
        let q = &self.candidates[self.index];
        let title = format!(
            " {} / {}   Candidate {} of {} ",
            q.id,
            q.topic,
            self.index + 1,
            self.candidates.len()
        );
        frame.render_widget(
            Paragraph::new(
                "RUST NYC / ORGANIZER PREVIEW / PROTOTYPE\nRead-only / reviewing exposes answers",
            ),
            header,
        );
        frame.render_widget(
            Tabs::new(["1 Question", "t Teaching trace", "r Reveal"])
                .select(match self.mode {
                    Mode::Question => 0,
                    Mode::Trace => 1,
                    Mode::Reveal => 2,
                })
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
                ),
            tabs,
        );
        let block = Block::default().borders(Borders::ALL).title(title);
        let inner = block.inner(content);
        let rows = self.rows(usize::from(inner.width));
        self.max_scroll = rows.len().saturating_sub(usize::from(inner.height));
        self.scroll = self.scroll.min(self.max_scroll);
        let visible: Vec<_> = rows
            .into_iter()
            .skip(self.scroll)
            .take(usize::from(inner.height))
            .collect();
        frame.render_widget(block, content);
        frame.render_widget(Paragraph::new(visible), inner);
        frame.render_widget(Paragraph::new(format!(
            "1 Question  t Trace  r Reveal  q Quit\n< > Step  [ ] Candidate  j/k Scroll\nPgUp/PgDn / scroll {} of {}", self.scroll, self.max_scroll
        )), footer);
    }
}

/// Render the real Ratatui widgets without opening an interactive terminal.
pub fn capture(app: &mut App, width: u16, height: u16) -> Result<String> {
    let mut terminal = Terminal::new(TestBackend::new(width, height))?;
    terminal.draw(|frame| app.draw(frame))?;
    let buffer = terminal.backend().buffer();
    let mut output = String::new();
    for y in 0..height {
        for x in 0..width {
            output.push_str(buffer[(x, y)].symbol());
        }
        output.push('\n');
    }
    Ok(output)
}

pub fn snapshot(keys: &[char], width: u16, height: u16) -> String {
    let mut app = App::load(&default_bank(), Some("q3")).expect("existing q3 fixture");
    for &key in keys {
        app.handle(key);
    }
    capture(&mut app, width, height).expect("in-memory rendering")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incomplete_trace_never_becomes_a_teaching_step() {
        for count in [0, 1] {
            let mut app = App::load(&default_bank(), Some("q3")).unwrap();
            app.candidates[app.index].trace.steps.truncate(count);
            app.handle('t');
            app.handle('>');
            let screen = capture(&mut app, 100, 36).unwrap();
            assert!(screen.contains("No teaching steps"));
            assert!(!screen.contains("Correct answer"));
        }
    }

    #[test]
    fn stdout_rows_are_suppressed_before_reveal() {
        let mut app = App::load(&default_bank(), Some("q3")).unwrap();
        app.candidates[app.index].trace.steps[0].values.push(Delta {
            name: "stdout".into(),
            was: "hidden-sentinel".into(),
            now: "hidden-sentinel".into(),
        });
        app.handle('t');
        assert!(!capture(&mut app, 100, 40)
            .unwrap()
            .contains("hidden-sentinel"));
    }

    #[test]
    fn missing_verification_is_not_invented() {
        let mut app = App::load(&default_bank(), Some("q3")).unwrap();
        let q = &mut app.candidates[app.index];
        q.correct = None;
        q.receipt.clear();
        q.legacy = false;
        app.handle('r');
        let screen = capture(&mut app, 100, 80).unwrap();
        assert!(screen.contains("Unavailable from the recorded evidence"));
        assert!(screen.contains("No complete receipt recorded"));
    }

    #[test]
    fn scroll_reaches_bottom_and_resize_preserves_navigation() {
        let mut app = App::load(&default_bank(), Some("q3")).unwrap();
        app.handle('r');
        capture(&mut app, 40, 14).unwrap();
        let max_scroll = app.max_scroll;
        assert!(max_scroll > 0);
        for _ in 0..200 {
            app.handle('J');
        }
        assert_eq!(app.scroll, max_scroll);
        let screen = capture(&mut app, 100, 36).unwrap();
        assert!(screen.contains("q Quit"));
        assert!(app.scroll <= app.max_scroll);
        app.handle('1');
        assert_eq!(app.scroll, 0);
    }
}
