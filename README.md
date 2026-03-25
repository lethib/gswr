# gswr

A fast, minimal terminal UI for switching Git branches — with GitHub PR titles shown inline.

<img src="https://8upload.com/image/59163cea43f374fa/gswr_demo.gif">

## Features

- Browse and switch local branches from an inline terminal panel
- Displays last commit date and message per branch
- Fetches open GitHub PR titles in the background (requires `GITHUB_TOKEN`)
- Keyboard-driven: no mouse needed

## Installation

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/lethib/gswr/releases/download/v1.0.1/gswr-installer.sh | sh
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
| `q` / `Ctrl+C` | Quit |

## GitHub PR integration

To display open PR titles next to each branch, export a GitHub personal access token:

```sh
export GITHUB_TOKEN=your_token_here
```

Without it, PR titles are simply not shown.

## Requirements

- macOS (Apple Silicon or Intel)
- Must be run from inside a Git repository

## License

MIT
