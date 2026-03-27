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
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/lethib/gswr/releases/download/v1.1.0/gswr-installer.sh | sh
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

Press `Ctrl+S` to delete all local branches whose PR has been merged (`M`) or closed (`C`). A confirmation prompt will appear — press `↵` to confirm or `c` to cancel.

## Requirements

- macOS (Apple Silicon or Intel)
- Must be run from inside a Git repository

## License

MIT
