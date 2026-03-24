use std::sync::mpsc::Sender;

use serde::Deserialize;

#[derive(Deserialize)]
struct PullRequest {
  title: String,
  head: PRHead,
}

#[derive(Deserialize)]
struct PRHead {
  #[serde(rename = "ref")]
  ref_name: String,
}

pub fn fetch_open_pr_titles(
  owner: &str,
  repo: &str,
  tx: Sender<(String, String)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
  let url = format!(
    "https://api.github.com/repos/{}/{}/pulls?state=open&per_page=100",
    owner, repo
  );

  let token = std::env::var("GITHUB_TOKEN")?;
  let repo_prs: Vec<PullRequest> = ureq::get(&url)
    .set("Authorization", &format!("Bearer {}", token))
    .set("User-Agent", "gswr")
    .set("Accept", "application/vnd.github+json")
    .set("X-GitHub-Api-Version", "2026-03-10")
    .call()?
    .into_json()?;

  for pr in repo_prs {
    if tx.send((pr.head.ref_name, pr.title)).is_err() {
      break;
    }
  }

  Ok(())
}
