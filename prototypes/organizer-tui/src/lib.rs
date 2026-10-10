//! Read-only question and authored teaching-trace preview.
use ratatui::{
    backend::TestBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Steps,
    Source,
    Narration,
}

const BG: Color = Color::Rgb(23, 28, 34);
const FG: Color = Color::Rgb(217, 222, 228);
const MUTED: Color = Color::Rgb(153, 162, 173);
const AMBER: Color = Color::Rgb(244, 199, 83);

pub struct App {
    candidates: Vec<Candidate>,
    index: usize,
    mode: Mode,
    step: usize,
    scroll: usize,
    max_scroll: usize,
    selected: usize,
    focus: Focus,
    source_scroll: usize,
    max_source_scroll: usize,
    help: bool,
    follow_source: bool,
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
            selected: 0,
            focus: Focus::Steps,
            source_scroll: 0,
            max_source_scroll: 0,
            help: false,
            follow_source: true,
        })
    }

    /// Returns false only for an explicit quit action.
    pub fn handle(&mut self, key: char) -> bool {
        if key == 'q' {
            return false;
        }
        if self.help {
            if matches!(key, '?' | '\u{1b}') {
                self.help = false;
            }
            return true;
        }
        let total = self.candidates[self.index].trace.steps.len();
        let before = (self.index, self.mode, self.step);
        match key {
            '?' => self.help = true,
            '\u{1b}' => {
                if self.mode == Mode::Reveal {
                    self.mode = Mode::Trace;
                    self.step = self.step.min(total.saturating_sub(2));
                    self.selected = self.step;
                    self.focus = Focus::Steps;
                } else if self.focus != Focus::Steps {
                    self.focus = Focus::Steps;
                } else {
                    self.mode = Mode::Question;
                }
            }
            '1' => self.mode = Mode::Question,
            '\t' if self.mode != Mode::Question => {
                self.focus = match self.focus {
                    Focus::Steps => Focus::Source,
                    Focus::Source => Focus::Narration,
                    Focus::Narration => Focus::Steps,
                };
            }
            't' => {
                self.mode = if self.mode == Mode::Trace {
                    Mode::Question
                } else {
                    Mode::Trace
                };
                self.step = self.step.min(total.saturating_sub(2));
                self.selected = self.step;
                self.focus = Focus::Steps;
            }
            'r' => {
                self.mode = Mode::Reveal;
                self.step = total.saturating_sub(1);
                self.focus = Focus::Narration;
            }
            '\n' if self.mode != Mode::Question && self.focus == Focus::Steps && total >= 2 => {
                self.step = self.selected.min(total - 2);
                self.mode = Mode::Trace;
            }
            '>' if self.mode != Mode::Question => {
                let reserve = if self.mode == Mode::Trace { 2 } else { 1 };
                self.step = (self.step + 1).min(total.saturating_sub(reserve));
                self.selected = self.step.min(total.saturating_sub(2));
            }
            '<' if self.mode != Mode::Question => {
                self.step = self.step.saturating_sub(1);
                self.selected = self.step.min(total.saturating_sub(2));
            }
            '[' | ']' => {
                let count = self.candidates.len();
                self.index = (self.index + if key == ']' { 1 } else { count - 1 }) % count;
                self.mode = Mode::Question;
                self.step = 0;
                self.selected = 0;
                self.focus = Focus::Steps;
            }
            'j' | 'k' | 'J' | 'K' if self.mode != Mode::Question && self.focus == Focus::Steps => {
                let delta = if matches!(key, 'J' | 'K') { 10 } else { 1 };
                self.selected = if matches!(key, 'j' | 'J') {
                    (self.selected + delta).min(total.saturating_sub(2))
                } else {
                    self.selected.saturating_sub(delta)
                };
            }
            'j' | 'k' | 'J' | 'K' if self.mode != Mode::Question && self.focus == Focus::Source => {
                let delta = if matches!(key, 'J' | 'K') { 10 } else { 1 };
                self.source_scroll = if matches!(key, 'j' | 'J') {
                    (self.source_scroll + delta).min(self.max_source_scroll)
                } else {
                    self.source_scroll.saturating_sub(delta)
                };
            }
            'j' => self.scroll = (self.scroll + 1).min(self.max_scroll),
            'k' => self.scroll = self.scroll.saturating_sub(1),
            'J' => self.scroll = (self.scroll + 10).min(self.max_scroll),
            'K' => self.scroll = self.scroll.saturating_sub(10),
            _ => {}
        }
        if before != (self.index, self.mode, self.step) {
            self.scroll = 0;
            self.source_scroll = 0;
            self.follow_source = true;
        }
        true
    }

    fn rows(&self, width: usize, with_source: bool) -> Vec<Line<'static>> {
        let q = &self.candidates[self.index];
        let mut rows = Vec::new();
        let normal = Style::default().fg(FG).bg(BG);
        let heading = normal.add_modifier(Modifier::BOLD);
        let muted = normal.fg(MUTED);
        let active = normal.fg(BG).bg(FG).add_modifier(Modifier::BOLD);
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
        if with_source {
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
        }
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

    fn source_rows(&self, width: usize) -> Vec<Line<'static>> {
        let q = &self.candidates[self.index];
        let step = if self.mode == Mode::Reveal || q.trace.steps.len() >= 2 {
            q.trace.steps.get(self.step)
        } else {
            None
        };
        let mut rows = vec![Line::raw("")];
        for (i, source) in q.source.lines().enumerate() {
            let number = i + 1;
            let marked = step.is_some_and(|s| s.lines.contains(&number));
            let mut style = Style::default().fg(FG).bg(BG);
            if marked {
                style = style.fg(BG).bg(FG).add_modifier(Modifier::BOLD);
            } else if step.is_some_and(|s| number < s.focus[0] || number > s.focus[1]) {
                style = style.fg(MUTED);
            }
            let safe: String = source
                .chars()
                .map(|c| if c.is_control() { ' ' } else { c })
                .collect();
            let prefix = format!("{} {number:2}  ", if marked { ">" } else { " " });
            for (part, text) in textwrap::wrap(&safe, width.saturating_sub(6).max(1))
                .iter()
                .enumerate()
            {
                rows.push(Line::styled(
                    format!("{}{}", if part == 0 { &prefix } else { "      " }, text),
                    style,
                ));
            }
        }
        rows
    }

    fn draw_steps(&self, frame: &mut Frame<'_>, area: Rect) {
        let focused = self.focus == Focus::Steps;
        let block = Block::default()
            .borders(Borders::ALL)
            .title(if focused {
                " Steps / FOCUS "
            } else {
                " Steps "
            })
            .border_style(Style::default().fg(if focused { AMBER } else { MUTED }));
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [list_area, reveal_area] =
            Layout::vertical([Constraint::Min(0), Constraint::Length(3)]).areas(inner);
        let q = &self.candidates[self.index];
        let teaching_count = q.trace.steps.len().saturating_sub(1);
        // Short navigation labels for the demonstration question; the teaching
        // content itself always comes from the bank. Other candidates use notes.
        let demo_labels = [
            "Build the vector",
            "Compare neighbours",
            "Middle pair",
            "Last pair",
            "Pause here",
        ];
        let items: Vec<_> = q
            .trace
            .steps
            .iter()
            .take(teaching_count)
            .enumerate()
            .map(|(i, step)| {
                let label = if q.id == "q3" {
                    demo_labels.get(i).copied().unwrap_or("Teaching step")
                } else {
                    step.note.split('.').next().unwrap_or("Teaching step")
                };
                ListItem::new(vec![
                    Line::raw(format!("{} {}", i + 1, label)),
                    Line::raw(""),
                ])
            })
            .collect();
        if teaching_count == 0 {
            frame.render_widget(Paragraph::new("No teaching steps"), list_area);
        } else {
            let mut state =
                ListState::default().with_selected(Some(self.selected.min(teaching_count - 1)));
            frame.render_stateful_widget(
                List::new(items)
                    .highlight_symbol("> ")
                    .highlight_style(Style::default().fg(AMBER).add_modifier(Modifier::BOLD)),
                list_area,
                &mut state,
            );
        }
        frame.render_widget(
            Paragraph::new(format!("{} Reveal / press r", q.trace.steps.len()))
                .style(Style::default().fg(if self.mode == Mode::Reveal {
                    AMBER
                } else {
                    MUTED
                }))
                .block(Block::default().borders(Borders::TOP)),
            reveal_area,
        );
    }

    fn draw_pane(&mut self, frame: &mut Frame<'_>, area: Rect, source: bool) {
        let focused = self.focus
            == if source {
                Focus::Source
            } else {
                Focus::Narration
            };
        let title = if source {
            "Source (Rust)"
        } else {
            "Teaching walkthrough"
        };
        let title = format!(" {title}{} ", if focused { " / FOCUS" } else { "" });
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .border_style(Style::default().fg(if focused { AMBER } else { MUTED }));
        let inner = block.inner(area);
        let rows = if source {
            self.source_rows(usize::from(inner.width))
        } else {
            self.rows(usize::from(inner.width), false)
        };
        let maximum = rows.len().saturating_sub(usize::from(inner.height));
        let offset = if source {
            self.max_source_scroll = maximum;
            if self.follow_source {
                if let Some(active) = rows.iter().position(|row| row.style.bg == Some(FG)) {
                    self.source_scroll = active.saturating_sub(1).min(maximum);
                }
                self.follow_source = false;
            }
            self.source_scroll = self.source_scroll.min(maximum);
            self.source_scroll
        } else {
            self.max_scroll = maximum;
            self.scroll = self.scroll.min(maximum);
            self.scroll
        };
        frame.render_widget(block, area);
        frame.render_widget(
            Paragraph::new(
                rows.into_iter()
                    .skip(offset)
                    .take(usize::from(inner.height))
                    .collect::<Vec<_>>(),
            ),
            inner,
        );
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        frame.render_widget(Block::default().style(Style::default().fg(FG).bg(BG)), area);
        if area.width < 40 || area.height < 14 {
            frame.render_widget(Paragraph::new("Resize to 40 x 14 or larger. q Quit"), area);
            return;
        }
        let [header, content, footer] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .areas(area);
        let q = &self.candidates[self.index];
        let mode = match self.mode {
            Mode::Question => "Question preview",
            Mode::Trace => "Teaching trace",
            Mode::Reveal => "Reveal",
        };
        frame.render_widget(
            Paragraph::new(format!(
                "RUST NYC / ORGANIZER PREVIEW / PROTOTYPE\n{} / {} / {mode}",
                q.id, q.topic
            )),
            header,
        );
        if self.mode == Mode::Question {
            let block = Block::default().borders(Borders::ALL).title(" Question ");
            let inner = block.inner(content);
            let rows = self.rows(usize::from(inner.width), true);
            self.max_scroll = rows.len().saturating_sub(usize::from(inner.height));
            self.scroll = self.scroll.min(self.max_scroll);
            frame.render_widget(block, content);
            frame.render_widget(
                Paragraph::new(
                    rows.into_iter()
                        .skip(self.scroll)
                        .take(usize::from(inner.height))
                        .collect::<Vec<_>>(),
                ),
                inner,
            );
        } else if area.width >= 80 {
            let [rail, body] =
                Layout::horizontal([Constraint::Percentage(28), Constraint::Percentage(72)])
                    .areas(content);
            let [source, narration] =
                Layout::vertical([Constraint::Percentage(42), Constraint::Percentage(58)])
                    .areas(body);
            self.draw_steps(frame, rail);
            self.draw_pane(frame, source, true);
            self.draw_pane(frame, narration, false);
        } else {
            match self.focus {
                Focus::Steps => self.draw_steps(frame, content),
                Focus::Source => self.draw_pane(frame, content, true),
                Focus::Narration => self.draw_pane(frame, content, false),
            }
        }
        let hints = if self.mode == Mode::Question {
            "Up/Down or j/k Scroll   t Teaching trace   ? Help"
        } else if self.focus == Focus::Steps {
            "Up/Down or j/k Select step   Enter Jump   Tab Focus   Esc Back"
        } else {
            "Up/Down or j/k Scroll pane   PgUp/PgDn Page   Tab Focus   Esc Back"
        };
        let footer_text = if area.width < 80 {
            "1 Question  t Trace  r Reveal  q Quit\nTab Pane  j/k Move  Enter Jump  ? Help\n< > Step  [ ] Candidate  Esc Back".to_string()
        } else {
            format!("{hints}\n1 Question   Left/Right Step   r Reveal   [ ] Candidate   ? Help   q Quit")
        };
        frame.render_widget(Paragraph::new(footer_text), footer);
        if self.help {
            let [_, row, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(12),
                Constraint::Fill(1),
            ])
            .areas(area);
            let [_, modal, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(area.width.min(76)),
                Constraint::Fill(1),
            ])
            .areas(row);
            frame.render_widget(Clear, modal);
            frame.render_widget(Paragraph::new("1 Question    t Teaching trace\nTab: focus steps, source, narration\nj/k or arrows: select or scroll\nEnter: jump to selected teaching step\nLeft/Right: step immediately\nr: reveal the final step and answer\n[ / ]: previous / next question\nEsc: back or close help    q: quit\n? or Esc closes this help")
                .style(Style::default().fg(FG).bg(BG))
                .block(Block::default().borders(Borders::ALL).title(" Keyboard help ").border_style(Style::default().fg(AMBER))), modal);
        }
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
