# gitviz

A fast terminal TUI for visualizing Git commit history — lane-based graph, HEAD/refs labels, commit details, and fuzzy search.

## Install

```bash
# From the project directory
cargo install --path .

# Or run directly
cargo run -- --max 200
```

## Usage

```bash
# Current directory (shows all branches, last 200 commits)
gitviz

# Specific repository
gitviz --repo /path/to/repo

# Limit to 50 commits
gitviz --max 50

# Only show HEAD history (not all branches)
gitviz --all false

# Exclude commits older than HEAD~500
gitviz --since HEAD~500

# Combine options
gitviz --repo ~/projects/myapp --max 100 --since HEAD~200
```

## Key Bindings

| Key            | Action                              |
| -------------- | ----------------------------------- |
| `j` / `↓`      | Move selection down                 |
| `k` / `↑`      | Move selection up                   |
| `g` / `Home`   | Jump to top (newest commit)         |
| `G` / `End`    | Jump to bottom (oldest commit)      |
| `Enter`        | Toggle details panel expand/collapse|
| `/`            | Enter filter mode (search by message)|
| `Esc`          | Clear filter, return to normal mode |
| `q`            | Quit                                |

## Graph Legend

```
● abc1234  [HEAD -> main]  Normal commit
◎ def5678  [feature]       Merge commit
│           (lane continuation)
```

Lanes are colour-coded so parallel branches are easy to follow.

## Project Structure

```
src/
  main.rs          Entry point, event loop
  cli.rs           CLI argument parsing (clap)
  app.rs           Application state & navigation logic
  git/
    model.rs       Commit and Refs data types
    commands.rs    git subprocess wrappers
    parser.rs      Parse git log / show-ref output
    mod.rs         load_commits(), load_refs(), check_repo()
  graph/
    lanes.rs       Lane assignment algorithm
    render.rs      Graph prefix string builder
    mod.rs
  ui/
    view.rs        Top-level ratatui layout
    widgets.rs     Individual panel renderers
    mod.rs
  util/
    fmt.rs         Timestamp formatting, short hash
    mod.rs
```
