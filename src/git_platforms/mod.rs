use ratatui::style::Color;
use std::sync::mpsc::Sender;

use crate::{app::BranchPRUpdate, git::branch::PRStatus};

pub mod github;

pub trait GitPlatform {
  fn pr_status_color(pr_status: &PRStatus) -> Color;
  fn fetch_open_pr_titles(owner: &str, repo: &str, tx: Sender<BranchPRUpdate>);
}
