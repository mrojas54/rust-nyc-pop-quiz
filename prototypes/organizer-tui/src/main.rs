use crossterm::event::{
    self, Event, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
    PushKeyboardEnhancementFlags,
};
use organizer_preview::{capture, default_bank, App, Result};
use std::{
    env,
    io::{self, IsTerminal},
    path::PathBuf,
};

// A paired push/pop restores the prior keyboard protocol on every normal/error exit.
struct KeyboardReporting(bool);
impl KeyboardReporting {
    fn enable() -> Self {
        let supported = crossterm::terminal::supports_keyboard_enhancement().unwrap_or(false);
        let enabled = supported
            && crossterm::execute!(
                io::stdout(),
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                )
            )
            .is_ok();
        Self(enabled)
    }
}
impl Drop for KeyboardReporting {
    fn drop(&mut self) {
        if self.0 {
            let _ = crossterm::execute!(io::stdout(), PopKeyboardEnhancementFlags);
        }
    }
}

fn main() -> Result<()> {
    let mut bank = default_bank();
    let mut question = None;
    let mut snapshot = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bank" => bank = PathBuf::from(args.next().ok_or("--bank needs a path")?),
            "--question" => question = Some(args.next().ok_or("--question needs an id")?),
            "--snapshot" => {
                snapshot = Some(
                    args.next()
                        .ok_or("--snapshot needs question, trace, or reveal")?,
                )
            }
            "--help" | "-h" => {
                println!("organizer-preview [--bank PATH] [--question ID] [--snapshot question|trace|reveal]\n\ns Schedule / 1 Question / t Trace / Left-Right Step / r then Enter Reveal / [ ] Candidate\nj-k or Up-Down Select or scroll / Enter Jump / Tab Focus / Esc Back / ? Help / q Quit");
                return Ok(());
            }
            _ => return Err(format!("Unknown argument: {arg}").into()),
        }
    }
    let mut app = App::load(&bank, question.as_deref())?;
    if let Some(mode) = snapshot {
        match mode.as_str() {
            "question" => {}
            "trace" => {
                app.handle('t');
            }
            "reveal" => {
                app.handle('r');
                app.handle('\n');
            }
            _ => return Err("--snapshot needs question, trace, or reveal".into()),
        }
        print!("{}", capture(&mut app, 100, 36)?);
        return Ok(());
    }
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("Open an interactive terminal, or use --snapshot question".into());
    }
    ratatui::run(|terminal| -> Result<()> {
        let _keyboard_reporting = KeyboardReporting::enable();
        loop {
            terminal.draw(|frame| app.draw(frame))?;
            if !event::poll(std::time::Duration::from_millis(100))? {
                continue;
            }
            if let Event::Key(key) = event::read()? {
                if !app.handle_event(key) {
                    break;
                }
            }
        }
        Ok(())
    })
}
