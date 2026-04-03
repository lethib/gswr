use std::{io::stdout, sync::mpsc, thread, time::Duration};

use crossterm::{
  event::{self, Event, KeyCode, KeyModifiers},
  terminal::{disable_raw_mode, enable_raw_mode},
};
use git2::Repository;
use ratatui::{Terminal, TerminalOptions, Viewport, prelude::CrosstermBackend};

use crate::{
  app::{App, GSWRActions},
  git::GSWRGitActions,
};

pub mod app;
pub mod git;
pub mod git_platforms;
pub mod ui;

fn main() -> Result<(), GSWRError> {
  let current_repo = Repository::discover(".")?;
  let branches = current_repo.list_branches()?;

  let pr_rx = match current_repo.extract_owner_repo() {
    Ok((owner, repo)) => {
      let (sender, receiver) = mpsc::channel();
      thread::spawn(move || git_platforms::github::fetch_open_pr_titles(&owner, &repo, sender));
      Some(receiver)
    }
    Err(_) => None,
  };

  let height = (branches.len() as u16 + 4).min(20).max(6);
  let mut app = App::new(branches, pr_rx);

  enable_raw_mode()?;
  let mut terminal = Terminal::with_options(
    CrosstermBackend::new(stdout()),
    TerminalOptions {
      viewport: Viewport::Inline(height),
    },
  )?;

  run_loop(&mut terminal, &mut app, &current_repo)?;

  terminal.clear()?;
  disable_raw_mode()?;

  Ok(())
}

fn run_loop(
  terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
  app: &mut App,
  repo: &Repository,
) -> Result<(), GSWRError> {
  loop {
    app.drain_pr_updates();
    if app.pr_rx.is_some() {
      app.throbber_state.calc_next();
    }
    terminal.draw(|frame| ui::draw(frame, app))?;

    if event::poll(Duration::from_millis(50))? {
      if let Event::Key(pressed_key) = event::read()? {
        if app.error_message.is_some() {
          app.error_message = None
        }

        if app.confirming_sync {
          match (pressed_key.code, pressed_key.modifiers) {
            (KeyCode::Enter, _) => {
              app.sync(repo);
              app.confirming_sync = false;
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,
            (KeyCode::Char('c'), _) => {
              app.confirming_sync = false;
            }
            _ => {}
          }
        } else {
          match (pressed_key.code, pressed_key.modifiers) {
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,

            (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.prev(),
            (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.next(),

            (KeyCode::Enter, _) => match app.confirm() {
              GSWRActions::Checkout(branch_name) => {
                repo.checkout(&branch_name)?;
                break;
              }
              GSWRActions::Quit => break,
              GSWRActions::None => {}
            },

            (KeyCode::Char('D'), _) => app.delete_selected_branch(repo),

            (KeyCode::Char('s'), KeyModifiers::CONTROL) => app.confirming_sync = true,

            _ => {}
          }
        }
      }
    }
  }

  Ok(())
}

#[derive(Clone)]
pub enum GSWRError {
  #[allow(non_camel_case_types)]
  PR_NOT_FOUND,
  Custom(String),
}

impl std::fmt::Debug for GSWRError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      GSWRError::PR_NOT_FOUND => write!(f, "❌ PR not found"),
      GSWRError::Custom(msg) => write!(f, "❌ {}", msg),
    }
  }
}

impl std::fmt::Display for GSWRError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      GSWRError::PR_NOT_FOUND => write!(f, "❌ PR not found"),
      GSWRError::Custom(msg) => write!(f, "❌ {}", msg),
    }
  }
}

impl From<git2::Error> for GSWRError {
  fn from(value: git2::Error) -> Self {
    GSWRError::Custom(value.message().to_string())
  }
}

impl From<std::io::Error> for GSWRError {
  fn from(value: std::io::Error) -> Self {
    GSWRError::Custom(value.to_string())
  }
}
