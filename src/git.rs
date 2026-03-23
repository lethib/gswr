use chrono::{DateTime, Local, TimeZone};
use git2::{BranchType, Repository};

pub struct BranchInfo {
  pub name: String,
  pub is_current: bool,
  pub last_commit_date: Option<DateTime<Local>>,
  pub last_commit_msg: Option<String>,
}

pub fn list_branches(repo: &Repository) -> Result<Vec<BranchInfo>, git2::Error> {
  let mut local_branches = repo
    .branches(Some(BranchType::Local))?
    .enumerate()
    .map(|(_i, branch)| -> Result<BranchInfo, git2::Error> {
      let branch = branch?.0;

      let branch_name = branch
        .name()?
        .ok_or(git2::Error::from_str("unnamed_branch"))?
        .to_string();
      let last_commit_date = Local
        .timestamp_opt(branch.get().peel_to_commit()?.time().seconds(), 0)
        .single();
      let last_commit_msg = branch
        .get()
        .peel_to_commit()?
        .summary()
        .map(|s| s.to_string());

      Ok(BranchInfo {
        name: branch_name,
        is_current: branch.is_head(),
        last_commit_date,
        last_commit_msg,
      })
    })
    .collect::<Result<Vec<BranchInfo>, _>>()?;

  local_branches.sort_by(|a, b| {
    b.is_current
      .cmp(&a.is_current)
      .then_with(|| b.last_commit_date.cmp(&a.last_commit_date))
  });

  Ok(local_branches)
}
