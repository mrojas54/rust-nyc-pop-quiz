use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use organizer_preview::{capture, default_bank, App, Result};
use std::{
    env,
    io::{self, IsTerminal},
    path::PathBuf,
};

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
                println!("organizer-preview [--bank PATH] [--question ID] [--snapshot question|trace|reveal]\n\n1 Question / t Trace / arrows Step / r Reveal / [ ] Candidate / j k Scroll / q Quit");
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
        loop {
            terminal.draw(|frame| app.draw(frame))?;
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    break;
                }
                let action = match key.code {
                    KeyCode::Char(c) => Some(c),
                    KeyCode::Left => Some('<'),
                    KeyCode::Right => Some('>'),
                    KeyCode::Up => Some('k'),
                    KeyCode::Down => Some('j'),
                    KeyCode::PageUp => Some('K'),
                    KeyCode::PageDown => Some('J'),
                    KeyCode::Tab => Some('\t'),
                    KeyCode::Esc => Some('q'),
                    _ => None,
                };
                if action.is_some_and(|key| !app.handle(key)) {
                    break;
                }
            }
        }
        Ok(())
    })
}
