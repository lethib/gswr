use chrono::{DateTime, Local};
use git2::{BranchType, Repository};

use crate::GSWRError;

#[derive(Clone, PartialEq)]
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

#[derive(Clone)]
pub struct BranchInfo {
  pub name: String,
  pub is_current: bool,
  pub is_main: bool,
  pub last_commit_date: Option<DateTime<Local>>,
  pub last_commit_msg: Option<String>,
  pub pr: PRResult,
}

pub(super) const MAIN_DEFAULT_BRANCH_NAMES: [&'static str; 4] =
  ["main", "master", "develop", "trunk"];

fn delete_branch(repo: &Repository, branch_name: &str) -> Result<(), git2::Error> {
  let mut branch_to_delete = repo.find_branch(branch_name, BranchType::Local)?;

  if branch_to_delete.is_head() {
    return Err(git2::Error::from_str("cannot delete current branch"));
  }

  branch_to_delete.delete()?;

  Ok(())
}

impl BranchInfo {
  pub fn delete(&self, repo: &Repository, safe_delete: bool) -> Result<(), git2::Error> {
    if self.is_main {
      return Err(git2::Error::from_str("cannot delete main branch"));
    }

    if self.is_current {
      return Err(git2::Error::from_str("cannot delete current branch"));
    }

    match &self.pr {
      Ok(pr) => {
        if pr.as_ref().is_some_and(|pr| pr.status == PRStatus::OPENED) {
          return Err(git2::Error::from_str(
            "cannot delete branch linked to an opened PR",
          ));
        }
      }
      Err(GSWRError::PR_NOT_FOUND) => {
        if safe_delete {
          return Err(git2::Error::from_str(
            "cannot delete branch not linked to a PR",
          ));
        }
      }
      Err(_) => {
        return Err(git2::Error::from_str(
          "cannot delete branch with an error on a PR",
        ));
      }
    }

    delete_branch(repo, &self.name)
  }
}
