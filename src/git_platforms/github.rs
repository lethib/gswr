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

  let client = reqwest::blocking::Client::new();
  let repo_prs = client
    .get(&url)
    .header(
      "Authorization",
      format!("Bearer {}", std::env::var("GITHUB_TOKEN")?),
    )
    .header("User-Agent", "gswr")
    .header("Accept", "application/vnd.github+json")
    .header("X-GitHub-Api-Version", "2026-03-10")
    .send()
    .and_then(|response| response.json::<Vec<PullRequest>>())?;

  for pr in repo_prs {
    if tx.send((pr.head.ref_name, pr.title)).is_err() {
      break;
    }
  }

  Ok(())
}
