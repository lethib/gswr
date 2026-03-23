use chrono::{DateTime, Local, TimeZone};
use git2::{BranchType, Repository};

pub struct BranchInfo {
  pub name: String,
  pub is_current: bool,
  pub last_commit_date: Option<DateTime<Local>>,
  pub last_commit_msg: Option<String>,
}

pub trait GSWRGitActions {
  fn list_branches(&self) -> Result<Vec<BranchInfo>, git2::Error>;
  fn checkout(&self, branch_name: &str) -> Result<(), git2::Error>;
}

impl GSWRGitActions for Repository {
  fn list_branches(&self) -> Result<Vec<BranchInfo>, git2::Error> {
    let mut local_branches = self
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

  fn checkout(&self, branch_name: &str) -> Result<(), git2::Error> {
    let (git_object, git_reference) = self.revparse_ext(branch_name)?;

    match git_reference {
      Some(reference) => {
        self.checkout_tree(&git_object, None)?;
        self.set_head(
          reference
            .name()
            .ok_or(git2::Error::from_str("no_name_on_git_reference"))?,
        )
      }
      None => Err(git2::Error::from_str("no_git_reference")),
    }
  }
}
