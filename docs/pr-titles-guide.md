# Guide : Afficher le titre de PR par branche (async)

## Vue d'ensemble

Le code actuel utilise `event::read()` qui est **bloquant** — le programme attend qu'une touche soit pressée sans rien faire d'autre. Pour fetcher les PRs en arrière-plan, il faut deux changements :

1. **Un thread dédié** qui appelle l'API GitHub **une seule fois au démarrage**, envoie les résultats via un channel `mpsc`, puis se termine
2. **Remplacer `event::read()`** par `event::poll()` avec un timeout — uniquement pour que l'UI puisse se mettre à jour quand les résultats arrivent, **pas pour refetcher**

> **Pourquoi `poll()` est nécessaire ?**
> Sans lui, `event::read()` bloquerait le thread principal indéfiniment jusqu'à la prochaine touche pressée. Si les résultats GitHub arrivent pendant ce blocage, l'UI ne se mettrait à jour qu'à ta prochaine pression de touche — l'utilisateur verrait les `⟳` pendant plusieurs secondes sans raison. Le `poll(50ms)` force le loop à se réveiller régulièrement pour vérifier le channel et redessiner si de nouveaux titres sont disponibles. Le fetch lui-même ne se produit qu'une seule fois.

```
Thread principal                   Thread PR fetcher
─────────────────                  ──────────────────
loop {                             spawn en arrière-plan
  drain_pr_updates()         ←──── envoie (branch_name, pr_title) via channel
  draw()
  poll(timeout: 50ms)
  handle key events
}
```

**Prérequis :** la variable d'environnement `GITHUB_TOKEN` doit être définie. Si elle est absente, l'app fonctionne normalement sans afficher les titres de PR.

---

## Étape 1 — Ajouter les dépendances

Dans `Cargo.toml` :

```toml
[dependencies]
chrono = "0.4.44"
crossterm = "0.29.0"
git2 = "0.20.4"
ratatui = "0.30.0"

# nouveaux
reqwest = { version = "0.12", features = ["blocking", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

> On utilise `reqwest` en mode **blocking** dans un thread séparé — pas besoin de tokio.

---

## Étape 2 — Mettre à jour `BranchInfo` dans `git.rs`

Ajouter un champ optionnel pour le titre de PR et l'initialiser à `None` :

```rust
pub struct BranchInfo {
    pub name: String,
    pub is_current: bool,
    pub last_commit_date: Option<DateTime<Local>>,
    pub last_commit_msg: Option<String>,
    pub pr_title: Option<String>, // None = chargement en cours, Some("") = pas de PR ouverte
}
```

Dans `list_branches`, lors de la construction du `BranchInfo` :

```rust
Ok(BranchInfo {
    name: branch_name,
    is_current: branch.is_head(),
    last_commit_date,
    last_commit_msg,
    pr_title: None, // sera rempli par le thread GitHub
})
```

Ajouter également une fonction pour extraire `owner/repo` depuis l'URL du remote `origin` :

```rust
/// Extrait ("owner", "repo") depuis l'URL du remote "origin".
/// Gère les formats SSH (git@github.com:owner/repo.git) et HTTPS.
pub fn get_github_owner_repo(repo: &Repository) -> Option<(String, String)> {
    let remote = repo.find_remote("origin").ok()?;
    let url = remote.url()?;

    let path = if url.contains("github.com:") {
        url.split("github.com:").nth(1)?
    } else {
        url.split("github.com/").nth(1)?
    };

    let path = path.trim_end_matches(".git");
    let mut parts = path.splitn(2, '/');
    let owner = parts.next()?.to_string();
    let repo_name = parts.next()?.to_string();

    Some((owner, repo_name))
}
```

---

## Étape 3 — Créer `src/github.rs`

Ce module tourne entièrement dans un thread séparé. Il fetche toutes les PRs ouvertes et envoie les résultats un par un via le channel.

```rust
use std::sync::mpsc::Sender;
use serde::Deserialize;

#[derive(Deserialize)]
struct PullRequest {
    title: String,
    head: PrHead,
}

#[derive(Deserialize)]
struct PrHead {
    #[serde(rename = "ref")]
    ref_name: String,
}

/// Fetche les PRs ouvertes et envoie (branch_name, pr_title) via `tx`.
/// Appelé dans un thread séparé — ne bloque pas le thread principal.
pub fn fetch_pr_titles(owner: &str, repo: &str, token: &str, tx: Sender<(String, String)>) {
    let url = format!(
        "https://api.github.com/repos/{}/{}/pulls?state=open&per_page=100",
        owner, repo
    );

    let client = reqwest::blocking::Client::new();
    let result = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2026-03-10")
        .header("User-Agent", "gswr")
        .send()
        .and_then(|r| r.json::<Vec<PullRequest>>());

    if let Ok(prs) = result {
        for pr in prs {
            // Si le receiver est déjà fermé (app quittée), on arrête silencieusement
            if tx.send((pr.head.ref_name, pr.title)).is_err() {
                break;
            }
        }
    }
}
```

Ne pas oublier de déclarer le module dans `main.rs` :

```rust
pub mod github;
```

---

## Étape 4 — Mettre à jour `App` dans `app.rs`

Ajouter le channel entrant et la méthode `drain_pr_updates` :

```rust
use std::sync::mpsc::{Receiver, TryRecvError};

pub struct App {
    pub selected: u8,
    pub local_branches: Vec<BranchInfo>,
    pub pr_rx: Option<Receiver<(String, String)>>,
}

impl App {
    pub fn new(branches: Vec<BranchInfo>, pr_rx: Option<Receiver<(String, String)>>) -> Self {
        App { selected: 0, local_branches: branches, pr_rx }
    }

    /// Consomme tous les messages disponibles dans le channel sans bloquer.
    /// À appeler au début de chaque itération du loop principal.
    /// Une fois le thread GitHub terminé (Disconnected), drope le receiver
    /// pour ne plus jamais le checker.
    pub fn drain_pr_updates(&mut self) {
        let Some(rx) = &self.pr_rx else { return };

        loop {
            match rx.try_recv() {
                Ok((branch_name, pr_title)) => {
                    if let Some(b) = self.local_branches.iter_mut().find(|b| b.name == branch_name) {
                        b.pr_title = Some(pr_title);
                    }
                }
                Err(TryRecvError::Disconnected) => {
                    // Thread terminé, plus rien à recevoir : on drop le receiver
                    self.pr_rx = None;
                    break;
                }
                Err(TryRecvError::Empty) => break, // Rien de disponible pour l'instant
            }
        }
    }

    // ... next(), prev(), confirm() inchangés
}
```

---

## Étape 5 — Mettre à jour `main.rs`

Spawn du thread et passage du receiver à `App`. Remplacer `event::read()` par `event::poll()` dans le loop :

```rust
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_repo = Repository::discover(".")?;
    let branches = current_repo.list_branches()?;

    // Lance le fetch en arrière-plan si GITHUB_TOKEN est défini et qu'on est sur GitHub
    let pr_rx = match (
        git::get_github_owner_repo(&current_repo),
        std::env::var("GITHUB_TOKEN").ok(),
    ) {
        (Some((owner, repo)), Some(token)) => {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || github::fetch_pr_titles(&owner, &repo, &token, tx));
            Some(rx)
        }
        _ => None,
    };

    let height = (branches.len() as u16 + 4).min(20).max(6);
    let mut app = App::new(branches, pr_rx);

    enable_raw_mode()?;
    let mut terminal = Terminal::with_options(
        CrosstermBackend::new(stdout()),
        TerminalOptions {
            viewport: Viewport::Inline(height),
        },
    )?;

    run_loop(&mut terminal, &mut app, &current_repo)?;

    terminal.clear()?;
    disable_raw_mode()?;

    Ok(())
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    repo: &Repository,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        app.drain_pr_updates(); // non-bloquant : consomme ce qui est dispo
        terminal.draw(|frame| ui::draw(frame, app))?;

        // poll avec timeout : si aucun événement clavier en 50ms,
        // on reboucle pour redessiner avec les éventuels nouveaux titres de PR
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(pressed_key) = event::read()? {
                match (pressed_key.code, pressed_key.modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => break,

                    (KeyCode::Up, _) | (KeyCode::Char('k'), _) => app.prev(),
                    (KeyCode::Down, _) | (KeyCode::Char('j'), _) => app.next(),

                    (KeyCode::Enter, _) => match app.confirm() {
                        GSWRActions::Checkout(branch_name) => {
                            repo.checkout(&branch_name)?;
                            break;
                        }
                        GSWRActions::Quit => break,
                        GSWRActions::None => {}
                    },
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
```

---

## Étape 6 — Afficher le titre dans l'UI (`ui.rs`)

Dans la fonction `draw`, modifier la construction des `ListItem` pour inclure le titre de PR :

```rust
let branches = app
    .local_branches
    .iter()
    .map(|branch| {
        let commit_date = branch
            .last_commit_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "---".to_string());

        let name = &branch.name;

        // Span affiché à droite du nom de branche
        let pr_span = match &branch.pr_title {
            None => Span::styled(" ⟳", Style::default().fg(MUTED)),          // en chargement
            Some(t) if t.is_empty() => Span::raw(""),                         // pas de PR ouverte
            Some(title) => Span::styled(
                format!("  {}", title),
                Style::default()
                    .fg(Color::Rgb(180, 150, 255))
                    .add_modifier(Modifier::ITALIC),
            ),
        };

        if branch.is_current {
            ListItem::new(Line::from(vec![
                Span::styled(" ● ", Style::default().fg(CURRENT).bold()),
                Span::styled(name.as_str(), Style::default().fg(CURRENT).bold()),
                pr_span,
                Span::styled(format!("  {}", commit_date), Style::default().fg(MUTED)),
            ]))
        } else {
            ListItem::new(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(name.as_str(), Style::default().fg(TEXT)),
                pr_span,
                Span::styled(format!("  {}", commit_date), Style::default().fg(MUTED)),
            ]))
        }
    })
    .collect::<Vec<ListItem>>();
```

---

## Résumé des fichiers à modifier

| Fichier | Nature du changement |
|---|---|
| `Cargo.toml` | Ajouter `reqwest`, `serde`, `serde_json` |
| `src/git.rs` | Ajouter `pr_title: None` dans `BranchInfo` + `get_github_owner_repo()` |
| `src/github.rs` | **Nouveau fichier** — fetch des PRs ouvertes via l'API GitHub |
| `src/app.rs` | Ajouter `pr_rx` dans `App` + méthode `drain_pr_updates()` |
| `src/main.rs` | Spawn du thread, `poll()` dans le loop à la place de `read()` |
| `src/ui.rs` | Afficher `pr_title` (ou `⟳`) dans chaque `ListItem` |

---

## Comportement attendu

- Au lancement, toutes les branches affichent `⟳` (chargement)
- Dès que le thread GitHub a répondu (~1-2s selon le réseau), les titres apparaissent branche par branche
- Si `GITHUB_TOKEN` n'est pas défini ou si le remote n'est pas sur GitHub, aucun `⟳` n'est affiché — l'app fonctionne exactement comme avant
- Si une branche n'a pas de PR ouverte, rien n'est affiché à côté de son nom
