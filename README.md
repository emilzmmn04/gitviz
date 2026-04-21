# gitviz

A fast, keyboard-driven terminal UI for visualizing Git commit history on macOS and Linux.

```text
● a1b2c3d  [HEAD -> main]  Add Summary / Files / Diff inspect tabs
│ ● 7c8d9e0                Add search-next navigation
◎ ─ ╮ e4f5a6b  [origin/main] Merge branch 'feature/search'
│ ● f1a2b3c                Improve compact merge rendering
● d4e5f6a  [tag: v0.1.0]   Initial MVP release
```

## Features

- **Compact commit graph** — lane-aware history with inline merge connectors
- **Ref labels** — HEAD, local branches, remote branches, and tags shown inline
- **Tabbed inspector** — `Summary`, `Files`, and `Diff` views for the selected commit
- **Commit actions** — copy the selected hash or open the commit on GitHub
- **Multi-field search** — live filter by subject, body, author, hash, email, or refs
- **Keyboard navigation** — vim-style (`j`/`k`), arrows, paging, and search result cycling
- **Lazy detail loading** — fast startup with on-demand file and diff inspection
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

### Debian package (release artifact)

```bash
sudo dpkg -i ./gitviz_<version>_amd64.deb
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

# Exclude commits reachable from a revision boundary
gitviz --exclude-reachable-from HEAD~500

# Combine flags
gitviz --repo ~/projects/myapp --max 100 --all false --exclude-reachable-from HEAD~500
```

## Key Bindings

| Key | Action |
|---|---|
| `j` / `↓` | Move selection down |
| `k` / `↑` | Move selection up |
| `g` / `Home` | Jump to newest commit (top) |
| `G` / `End` | Jump to oldest commit (bottom) |
| `Enter` | Toggle details panel expand / collapse |
| `Tab` | Cycle details tabs: `Summary` → `Files` → `Diff` |
| `Shift-Tab` | Cycle details tabs in reverse |
| `PageDown` | Scroll the active details tab down |
| `PageUp` | Scroll the active details tab up |
| `Ctrl-d` | Scroll the active details tab down by half a page |
| `Ctrl-u` | Scroll the active details tab up by half a page |
| `r` | Reload repository state |
| `/` | Enter search mode — filter by subject, body, author, hash, email, or refs |
| `n` | Jump to the next matching commit when a search filter is active |
| `N` | Jump to the previous matching commit when a search filter is active |
| `y` | Copy the selected commit hash to the clipboard |
| `o` | Open the selected commit on GitHub for supported `origin` remotes |
| `?` | Toggle the help overlay |
| `Esc` | Clear search filter, return to normal mode |
| `q` | Quit |

`Files` and `Diff` load lazily for the selected commit. Very large patches are truncated in the preview and shown with a truncation notice.

## CLI Options

| Flag | Default | Description |
|---|---|---|
| `--all` | `true` | Show all branches |
| `--max <N>` | `200` | Maximum commits to load |
| `--exclude-reachable-from <rev>` | — | Exclude commits reachable from this revision boundary |
| `--repo <path>` | `.` | Path to the git repository |
| `--no-color` | — | Disable coloured styling and use monochrome rendering |

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
├── app.rs           Application state: selection, filter, tabs, status, inspect cache
├── git/
│   ├── commands.rs  git subprocess wrappers (no shell, no libgit2)
│   ├── parser.rs    Parse git log, show-ref, and name-status output
│   ├── model.rs     Commit, refs, changed-file, and inspect-cache types
│   └── mod.rs       load_commits(), load_refs(), load_commit_inspect_data()
├── graph/
│   ├── lanes.rs     Lane assignment and compact merge connector layout
│   ├── render.rs    Graph row renderer (● │ ◎ ─ ╮ ╭)
│   └── mod.rs
├── ui/
│   ├── view.rs      Top-level ratatui layout
│   ├── widgets.rs   Graph list, tabbed details panel, help overlay renderers
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
Standard validation lives in `.github/workflows/ci.yml` and runs formatting, tests, and clippy on pushes and pull requests.
Package smoke coverage lives in `.github/workflows/package-smoke.yml`.
Release artifact dry-runs live in `.github/workflows/release-dry-run.yml`.

The documented release gate is in [docs/release-checklist.md](docs/release-checklist.md).

Optional repository secrets (only needed for the corresponding channel):

- `NPM_TOKEN` for npm publishing (`@emilzmmn04/gitviz`)
- `HOMEBREW_TAP_GITHUB_TOKEN` for updating `emilzmmn04/homebrew-tap`

## Railway Landing Page Prelaunch

The repository includes a static landing-page prelaunch workflow:

- `.github/workflows/prelaunch-checklist.yml`

It validates:

- Release workflow secret gates in `.github/workflows/release.yml`
- Package manager secrets
- Landing page availability (`200` + `text/html`) on Railway URL
- Optional custom domain HTTPS endpoint

You can run the checks locally:

```bash
# 1) Verify release workflow gates
./scripts/verify_release_workflow_gates.sh

# 2) Verify required release secrets and key material
export NPM_TOKEN=...
export HOMEBREW_TAP_GITHUB_TOKEN=...
./scripts/check_release_secrets.sh

# 3) Smoke-test Railway landing page
./scripts/check_landing_page.sh https://<your-railway-url>
# optional custom domain check
./scripts/check_landing_page.sh https://<your-railway-url> <your-custom-domain>
```

Or run the same checks in GitHub Actions:

- Open `Prelaunch Checklist` workflow
- Provide `landing_url`
- Optionally provide `custom_domain`

## Contributing

Contributions are welcome! Please open an issue before submitting a large PR.

```bash
git clone https://github.com/emilzmmn04/gitviz.git
cd gitviz
rustup component add rustfmt clippy
cargo test        # run all tests
cargo clippy      # lint
cargo fmt         # format
```

## Notes

- Supported platforms: macOS and Linux
- Supported install methods: Cargo, npm, Homebrew, and Debian release artifacts
- GitHub open action supports GitHub `origin` remotes only
- `--no-color` keeps the graph and UI readable in monochrome terminals
- The root repository is the only canonical source tree; the ignored `/gitviz/` path is not part of the build or release flow

## License

MIT — see [LICENSE](LICENSE)
