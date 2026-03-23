use std::io::{Stdout, stdout};

use crossterm::{
  event::{self, Event, KeyCode, KeyModifiers},
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use git2::Repository;
use ratatui::{Terminal, prelude::CrosstermBackend};

use crate::{
  app::{App, GSWActions},
  git::GSWGitActions,
};

pub mod app;
pub mod git;
pub mod ui;

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let current_repo = Repository::discover(".")?;
  let branches = current_repo.list_branches()?;
  let mut app = App::new(branches);
  let mut stdout = stdout();

  enable_raw_mode()?;
  execute!(stdout, EnterAlternateScreen)?;
  let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;

  run_loop(&mut terminal, &mut app, &current_repo)?;

  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

  Ok(())
}

fn run_loop(
  terminal: &mut Terminal<CrosstermBackend<Stdout>>,
  app: &mut App,
  repo: &Repository,
) -> Result<(), Box<dyn std::error::Error>> {
  loop {
    terminal.draw(|frame| ui::draw(frame, app))?;

    if let Event::Key(pressed_key) = event::read()? {
      match (pressed_key.code, pressed_key.modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,

        (KeyCode::Up, _) => app.prev(),
        (KeyCode::Down, _) => app.next(),

        (KeyCode::Enter, _) => match app.confirm() {
          GSWActions::Checkout(branch_name) => {
            repo.checkout(&branch_name)?;
            break;
          }
          GSWActions::Quit => break,
          GSWActions::None => {}
        },
        _ => {}
      }
    }
  }

  Ok(())
}
