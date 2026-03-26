use std::sync::mpsc::Sender;

use serde::Deserialize;

use crate::{
  GSWRError,
  app::ChannelReceiver,
  git::{PR, PRStatus},
};

#[derive(Deserialize)]
enum PRState {
  #[serde(rename = "open")]
  OPEN,
  #[serde(rename = "closed")]
  CLOSED,
}

#[derive(Deserialize)]
struct PullRequest {
  title: String,
  head: PRHead,
  state: PRState,
  merged_at: Option<String>,
}

#[derive(Deserialize)]
struct PRHead {
  #[serde(rename = "ref")]
  ref_name: String,
}

pub fn fetch_open_pr_titles(owner: &str, repo: &str, tx: Sender<ChannelReceiver>) {
  let url = format!(
    "https://api.github.com/repos/{}/{}/pulls?state=all&per_page=100",
    owner, repo
  );

  let token = match std::env::var("GITHUB_TOKEN") {
    Ok(t) => t,
    Err(e) => {
      let _ = tx.send(ChannelReceiver {
        branch_name: None,
        pr_result: Err(GSWRError::Custom(format!("GITHUB_TOKEN {}", e.to_string()))),
      });
      return;
    }
  };

  let response = match ureq::get(&url)
    .set("Authorization", &format!("Bearer {}", token))
    .set("User-Agent", "gswr")
    .set("Accept", "application/vnd.github+json")
    .set("X-GitHub-Api-Version", "2026-03-10")
    .call()
  {
    Ok(r) => r,
    Err(e) => {
      let _ = tx.send(ChannelReceiver {
        branch_name: None,
        pr_result: Err(GSWRError::Custom(e.to_string())),
      });
      return;
    }
  };

  let repo_prs: Vec<PullRequest> = match response.into_json() {
    Ok(prs) => prs,
    Err(e) => {
      let _ = tx.send(ChannelReceiver {
        branch_name: None,
        pr_result: Err(GSWRError::from(e)),
      });
      return;
    }
  };

  for pr in repo_prs {
    if tx
      .send(ChannelReceiver {
        branch_name: Some(pr.head.ref_name),
        pr_result: Ok(Some(PR {
          title: pr.title,
          status: match pr.state {
            PRState::OPEN => PRStatus::OPENED,
            PRState::CLOSED if pr.merged_at.is_some() => PRStatus::MERGED,
            PRState::CLOSED => PRStatus::CLOSED,
          },
        })),
      })
      .is_err()
    {
      break;
    }
  }
}
