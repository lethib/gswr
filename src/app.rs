use std::sync::mpsc::{Receiver, TryRecvError};

use crate::git::BranchInfo;

pub enum GSWRActions {
  Checkout(String),
  Quit,
  None,
}

pub struct App {
  pub local_branches: Vec<BranchInfo>,
  pub selected: u8,
  pub pr_rx: Option<Receiver<(String, String)>>,
}

impl App {
  pub fn new(branches: Vec<BranchInfo>, pr_rx: Option<Receiver<(String, String)>>) -> Self {
    App {
      local_branches: branches,
      selected: 0,
      pr_rx,
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
        Ok((branch_name, pr_title)) => {
          if let Some(branch) = self
            .local_branches
            .iter_mut()
            .find(|b| b.name == branch_name)
          {
            branch.pr_title = Some(pr_title)
          }
        }
        Err(TryRecvError::Disconnected) => {
          // Thread over: branches still None have no open PR
          for branch in self.local_branches.iter_mut() {
            if branch.pr_title.is_none() {
              branch.pr_title = Some(String::new());
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
