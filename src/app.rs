use crate::git::BranchInfo;

pub enum GSWActions {
  Checkout(String),
  Quit,
  None,
}

pub struct App {
  pub local_branches: Vec<BranchInfo>,
  pub selected: u8,
}

impl App {
  pub fn new(branches: Vec<BranchInfo>) -> Self {
    App {
      local_branches: branches,
      selected: 0,
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

  pub fn confirm(&self) -> GSWActions {
    match self.local_branches.get(self.selected as usize) {
      Some(branch) if branch.is_current => GSWActions::Quit,
      Some(branch) => GSWActions::Checkout(branch.name.clone()),
      None => GSWActions::Quit,
    }
  }
}
