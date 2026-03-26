use std::sync::mpsc::{Receiver, TryRecvError};

use crate::{
  GSWRError,
  git::{BranchInfo, PRResult},
};

pub enum GSWRActions {
  Checkout(String),
  Quit,
  None,
}

pub struct ChannelReceiver {
  pub branch_name: Option<String>,
  pub pr_result: PRResult,
}

pub struct App {
  pub local_branches: Vec<BranchInfo>,
  pub selected: u8,
  pub pr_rx: Option<Receiver<ChannelReceiver>>,
  pub throbber_state: throbber_widgets_tui::ThrobberState,
}

impl App {
  pub fn new(branches: Vec<BranchInfo>, pr_rx: Option<Receiver<ChannelReceiver>>) -> Self {
    App {
      local_branches: branches,
      selected: 0,
      pr_rx,
      throbber_state: throbber_widgets_tui::ThrobberState::default(),
    }
  }

  pub fn next(&mut self) {
    if !self.local_branches.is_empty() {
      self.selected = (self.selected + 1) % self.local_branches.len() as u8;
    }
  }

  pub fn prev(&mut self) {
    if !self.local_branches.is_empty() {
      self.selected = self.selected.saturating_sub(1);
    }
  }

  pub fn confirm(&self) -> GSWRActions {
    match self.local_branches.get(self.selected as usize) {
      Some(branch) if branch.is_current => GSWRActions::Quit,
      Some(branch) => GSWRActions::Checkout(branch.name.clone()),
      None => GSWRActions::Quit,
    }
  }

  pub fn drain_pr_updates(&mut self) {
    let Some(rx) = &self.pr_rx else { return };

    loop {
      match rx.try_recv() {
        Ok(msg) => match msg.branch_name {
          Some(branch_name) => {
            if let Some(branch) = self
              .local_branches
              .iter_mut()
              .find(|b| b.name == branch_name)
            {
              branch.pr = msg.pr_result;
            }
          }
          None => {
            self
              .local_branches
              .iter_mut()
              .for_each(|branch| branch.pr = msg.pr_result.clone());
          }
        },
        Err(TryRecvError::Disconnected) => {
          // Thread over: branches still None have no open PR
          for branch in self.local_branches.iter_mut() {
            if branch.pr.as_ref().is_ok_and(|b| b.is_none()) {
              branch.pr = Err(GSWRError::PR_NOT_FOUND);
            }
          }
          self.pr_rx = None;
          break;
        }
        Err(TryRecvError::Empty) => break,
      }
    }
  }
}
