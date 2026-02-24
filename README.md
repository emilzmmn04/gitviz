# gitviz

A fast, keyboard-driven terminal UI for visualizing Git commit history.

```
● a1b2c3d  [HEAD -> main]  Merge pull request #42: add lane colours
◎ e4f5a6b  [origin/main]   Merge branch 'feature/search'
│ ● 7c8d9e0                Add / filter mode for commit search
│ ● f1a2b3c                Fix lane allocation for octopus merges
│/
● d4e5f6a  [v0.1.0]        Initial MVP release
│ ● 7b8c9d1  [tag: v0.0.1] Proof of concept
│/
● 0e1f2a3b                 Initial commit
```

## Features

- **Lane-based commit graph** — parallel branches rendered as colour-coded columns
- **Ref labels** — HEAD, local branches, remote branches, and tags shown inline
- **Commit details panel** — full hash, author, date, and message on selection
- **Fuzzy search** — live filter commits by message substring
- **Keyboard navigation** — vim-style (`j`/`k`) and arrow keys
- **Fast** — single `git log` call, no libgit2 dependency
- **Works on macOS and Linux**

## Install

### Cargo (source install, Rust >= 1.70)

```bash
git clone https://github.com/emilzmmn04/gitviz.git
cd gitviz
cargo install --path .
```

### npm (prebuilt binary)

```bash
npm i -g @emilzmmn04/gitviz
```

### Homebrew (prebuilt binary)

```bash
brew tap emilzmmn04/tap
brew install gitviz
```

### APT (prebuilt binary)

```bash
sudo mkdir -p /etc/apt/keyrings
curl -fsSL https://emilzmmn04.github.io/gitviz/apt/keyrings/gitviz-archive-keyring.gpg \
  | sudo tee /etc/apt/keyrings/gitviz-archive-keyring.gpg >/dev/null
echo "deb [signed-by=/etc/apt/keyrings/gitviz-archive-keyring.gpg] https://emilzmmn04.github.io/gitviz/apt stable main" \
  | sudo tee /etc/apt/sources.list.d/gitviz.list >/dev/null
sudo apt update
sudo apt install gitviz
```

### Run without installing

```bash
cargo run -- --max 100
```

> Make sure `~/.cargo/bin` is on your `$PATH`.
> Add `export PATH="$HOME/.cargo/bin:$PATH"` to your `.zshrc` / `.bashrc` if needed.

## Usage

```bash
# Current directory — all branches, last 200 commits (default)
gitviz

# Specific repository
gitviz --repo /path/to/repo

# Limit commit count
gitviz --max 50

# HEAD branch only (skip other branches)
gitviz --all false

# Exclude history older than a revision
gitviz --since HEAD~500

# Combine flags
gitviz --repo ~/projects/myapp --max 100 --all false
```

## Key Bindings

| Key | Action |
|---|---|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `g` / `Home` | Jump to newest commit (top) |
| `G` / `End` | Jump to oldest commit (bottom) |
| `Enter` | Toggle details panel expand / collapse |
| `/` | Enter search mode — filter by commit message |
| `Esc` | Clear search filter, return to normal mode |
| `q` | Quit |

## CLI Options

| Flag | Default | Description |
|---|---|---|
| `--all` | `true` | Show all branches |
| `--max <N>` | `200` | Maximum commits to load |
| `--since <rev>` | — | Exclude commits reachable from this revision |
| `--repo <path>` | `.` | Path to the git repository |
| `--no-color` | — | Disable colours (flag accepted, TUI always uses colours) |

## Release Artifacts

Every `vX.Y.Z` tag publishes:

- `gitviz-v{version}-{target}.tar.gz`
- `gitviz-v{version}-{target}.sha256`
- `gitviz_{version}_amd64.deb`

Current binary targets:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

## Requirements

- Git ≥ 2.0 installed and on `$PATH`
- A real terminal (not piped / redirected)
- macOS or Linux

## Project Structure

```
src/
├── main.rs          Entry point, event loop, terminal setup
├── cli.rs           CLI argument parsing (clap)
├── app.rs           Application state: selection, filter, navigation
├── git/
│   ├── commands.rs  git subprocess wrappers (no shell, no libgit2)
│   ├── parser.rs    Parse git log and show-ref output
│   ├── model.rs     Commit and Refs data types
│   └── mod.rs       load_commits(), load_refs(), check_repo()
├── graph/
│   ├── lanes.rs     Lane assignment algorithm
│   ├── render.rs    Graph prefix string builder (● │ ◎)
│   └── mod.rs
├── ui/
│   ├── view.rs      Top-level ratatui layout
│   ├── widgets.rs   Graph list, details panel, filter bar renderers
│   └── mod.rs
└── util/
    ├── fmt.rs       Relative timestamps, short hash, ISO-8601
    └── mod.rs
```

## Dependencies

| Crate | Purpose |
|---|---|
| [ratatui](https://github.com/ratatui/ratatui) | Terminal UI framework |
| [crossterm](https://github.com/crossterm-rs/crossterm) | Cross-platform terminal control |
| [clap](https://github.com/clap-rs/clap) | CLI argument parsing |
| [anyhow](https://github.com/dtolnay/anyhow) | Error handling |

## Package Automation

Release automation lives in `.github/workflows/release.yml` and can publish all package channels from a single tag push.

Optional repository secrets (only needed for the corresponding channel):

- `NPM_TOKEN` for npm publishing (`@emilzmmn04/gitviz`)
- `HOMEBREW_TAP_GITHUB_TOKEN` for updating `emilzmmn04/homebrew-tap`
- `APT_GPG_PRIVATE_KEY` for signing APT metadata
- `APT_GPG_PASSPHRASE` when the APT key is passphrase-protected

## Contributing

Contributions are welcome! Please open an issue before submitting a large PR.

```bash
git clone https://github.com/emilzmmn04/gitviz.git
cd gitviz
cargo test        # run all tests
cargo clippy      # lint
cargo fmt         # format
```

## License

MIT — see [LICENSE](LICENSE)
