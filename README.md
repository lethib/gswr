# gswr

A fast, minimal terminal UI for switching Git branches — with GitHub PR status and titles shown inline.

<img src="https://8upload.com/image/1cb119feea7326fb/gswr_demo.gif">

## Features

- Browse and switch local branches from an inline terminal panel
- Displays last commit date and message per branch
- Fetches PR titles and statuses (open, merged, closed) in the background (requires `GITHUB_TOKEN`)
- Sync local branches: delete branches whose PR has been merged or closed with `Ctrl+S`
- Keyboard-driven: no mouse needed

## Installation

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/lethib/gswr/releases/download/v2.0.0/gswr-installer.sh | sh
```

> macOS only (Apple Silicon & Intel). Linux and Windows support coming soon.

## Usage

Run `gswr` from anywhere inside a Git repository:

```sh
gswr
```

| Key | Action |
|---|---|
| `↑` / `k` | Move up |
| `↓` / `j` | Move down |
| `↵` | Switch to selected branch |
| `Shift+D` | Delete selected branch (not allowed on main branch) |
| `Ctrl+S` | Sync branches (delete merged/closed) |
| `q` / `Ctrl+C` | Quit |

## GitHub PR integration

To display PR titles and statuses next to each branch, export a GitHub personal access token:

```sh
export GITHUB_TOKEN=your_token_here
```

Each branch displays a status indicator:

| Status | Meaning |
|---|---|
| `O` | Open PR |
| `M` | Merged PR |
| `C` | Closed PR |

Without `GITHUB_TOKEN`, PR information is not shown.

### Branch sync

Press `Ctrl+S` to delete all local branches whose PR has been merged (`M`) or closed (`C`). A confirmation prompt will appear:

- Press `↵` to confirm (safe mode: only deletes branches with a merged/closed PR)
- Press `⌥↵` (Alt+Enter) to also delete local branches with **no linked PR** — unsafe mode, skips branches without a PR status (the main branch is always protected)
- Press `c` to cancel

The main branch is always protected and cannot be deleted, whether individually or during sync.

## oh-my-zsh users

If you use oh-my-zsh with the `git` plugin, `gsw` is already aliased to `git switch`. You may want to alias `gswr` to a shorter name in your `.zshrc`:

```sh
alias gsw=gswr
```

This overrides the oh-my-zsh `gsw` alias with `gswr`. If you still need the original `git switch` shorthand, use `git switch` directly or pick a different alias.

## Requirements

- macOS (Apple Silicon or Intel)
- Must be run from inside a Git repository

## License

MIT
