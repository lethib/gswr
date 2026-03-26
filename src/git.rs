use chrono::{DateTime, Local, TimeZone};
use git2::{BranchType, Repository};

use crate::GSWRError;

#[derive(Clone)]
pub enum PRStatus {
  OPENED,
  MERGED,
  CLOSED,
}

#[derive(Clone)]
pub struct PR {
  pub title: String,
  pub status: PRStatus,
}

pub type PRResult = Result<Option<PR>, GSWRError>;

pub struct BranchInfo {
  pub name: String,
  pub is_current: bool,
  pub last_commit_date: Option<DateTime<Local>>,
  pub last_commit_msg: Option<String>,
  pub pr: PRResult,
}

pub trait GSWRGitActions {
  fn list_branches(&self) -> Result<Vec<BranchInfo>, git2::Error>;
  fn checkout(&self, branch_name: &str) -> Result<(), git2::Error>;
  fn extract_owner_repo(&self) -> Result<(String, String), git2::Error>;
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
          pr: Ok(None),
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

  fn extract_owner_repo(&self) -> Result<(String, String), git2::Error> {
    let remote = self.find_remote("origin")?;

    let remote_url = remote
      .url()
      .ok_or(git2::Error::from_str("no URL for remote"))?;

    let path = if remote_url.contains("github.com:") {
      remote_url.split("github.com:").nth(1)
    } else {
      remote_url.split("github.com/").nth(1)
    };

    let path = path
      .ok_or(git2::Error::from_str("no_path_found"))?
      .trim_end_matches(".git");
    let mut parts = path.splitn(2, '/');
    let owner = parts
      .next()
      .ok_or(git2::Error::from_str("no_owner_found"))?
      .to_string();
    let repo_name = parts
      .next()
      .ok_or(git2::Error::from_str("no_repo_name_found"))?
      .to_string();

    Ok((owner, repo_name))
  }
}
