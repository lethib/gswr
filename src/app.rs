use std::sync::mpsc::{Receiver, TryRecvError};

use git2::Repository;

use crate::{
  GSWRError,
  git::{BranchInfo, GSWRGitActions, PRResult, PRStatus},
};

pub enum GSWRActions {
  Checkout(String),
  Quit,
  None,
}

pub struct BranchPRUpdate {
  pub branch_name: Option<String>,
  pub pr_result: PRResult,
}

pub struct App {
  pub local_branches: Vec<BranchInfo>,
  pub selected: u8,
  pub pr_rx: Option<Receiver<BranchPRUpdate>>,
  pub throbber_state: throbber_widgets_tui::ThrobberState,
  pub confirming_sync: bool,
  pub error_message: Option<String>,
}

impl App {
  pub fn new(branches: Vec<BranchInfo>, pr_rx: Option<Receiver<BranchPRUpdate>>) -> Self {
    App {
      local_branches: branches,
      selected: 0,
      pr_rx,
      throbber_state: throbber_widgets_tui::ThrobberState::default(),
      confirming_sync: false,
      error_message: None,
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

  pub fn sync(&mut self, repo: &Repository) {
    let branches_to_delete = self
      .local_branches
      .iter()
      .filter(|branch| !branch.is_current)
      .filter(|branch| match &branch.pr {
        Ok(pr) => pr.as_ref().is_some_and(|defined_pr| {
          defined_pr.status == PRStatus::CLOSED || defined_pr.status == PRStatus::MERGED
        }),
        Err(_) => false,
      })
      .map(|b| b.name.clone())
      .collect::<Vec<String>>();

    for branch_to_delete in branches_to_delete {
      match repo.delete_branch(&branch_to_delete) {
        Ok(()) => self.local_branches.retain(|b| b.name != branch_to_delete),
        Err(error) => {
          self.error_message = Some(error.to_string());
          break;
        }
      }
    }
  }

  pub fn delete_selected_branch(&mut self, repo: &Repository) {
    let Some(branch_to_delete) = self.local_branches.get(self.selected as usize) else {
      self.error_message = Some("local branch not found".to_string());
      return;
    };

    match &branch_to_delete.pr {
      Ok(pr) => {
        if pr.as_ref().is_some_and(|pr| pr.status == PRStatus::OPENED) {
          self.error_message = Some("cannot delete branch linked to an opened PR".to_string());
          return;
        }
      }
      Err(GSWRError::PR_NOT_FOUND) => {
        self.error_message = Some("cannot delete branch not linked to a PR".to_string());
        return;
      }
      Err(_) => return,
    }

    let branch_name = branch_to_delete.name.clone();

    match repo.delete_branch(&branch_name) {
      Ok(()) => self.local_branches.retain(|b| b.name != branch_name),
      Err(error) => self.error_message = Some(error.to_string()),
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
