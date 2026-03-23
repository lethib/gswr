use git2::Repository;

use crate::git::list_branches;

pub mod app;
pub mod git;
pub mod ui;

fn main() -> Result<(), git2::Error> {
  let current_repo = Repository::discover(".")?;
  let branches = list_branches(&current_repo)?;
  for branch in branches {
    let prefix = if branch.is_current { "* " } else { "  " };
    let date = branch
      .last_commit_date
      .map(|d| d.format("%Y-%m-%d").to_string())
      .unwrap_or_else(|| "unknown".to_string());
    let msg = branch
      .last_commit_msg
      .unwrap_or_else(|| "no message".to_string());
    println!("{}{} ({}) {}", prefix, branch.name, date, msg);
  }
  Ok(())
}
