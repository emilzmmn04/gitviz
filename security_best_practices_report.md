# Security and Reliability Report (gitviz)

Date: 2026-02-24
Scope: `/Users/emilzimmermann/gitviz` (root crate) and `/Users/emilzimmermann/gitviz/gitviz` (nested duplicate crate)

## Executive Summary

Core unit tests pass, and baseline CLI smoke checks behave correctly. One high-impact denial-of-service condition was identified in timestamp formatting for extreme commit dates, and one medium integrity issue was identified in commit parsing behavior for malformed commit metadata.

## Test Execution Summary

### Automated tests

- `cargo test` in `/Users/emilzimmermann/gitviz`: **PASS** (13/13)
- `cargo test` in `/Users/emilzimmermann/gitviz/gitviz`: **PASS** (13/13)
- `cargo check --release` in `/Users/emilzimmermann/gitviz`: **PASS**

### Smoke tests

- `cargo run -- --help`: **PASS**
- `cargo run -- --repo /tmp/does-not-exist-gitviz-smoke --max 10`: **PASS** (expected error path)
- `cargo run -- --repo /Users/emilzimmermann/gitviz --max 10 --since definitely_not_a_ref`: **PASS** (expected git error surfaced)
- PTY interactive run against `/Users/emilzimmermann/gitviz` (`q` quit path): **PASS**

### Tooling gaps

- `cargo clippy --all-targets --all-features -- -D warnings`: **NOT RUN** (`cargo-clippy` not installed)
- `cargo fmt --all -- --check`: **NOT RUN** (`cargo-fmt` / `rustfmt` not installed)
- `cargo audit`: **NOT RUN** (`cargo-audit` not installed)

## Findings

## [HIGH] F-001: CPU denial-of-service via extreme commit timestamps

Impact: A repository containing an extreme timestamp can cause the TUI to hang while rendering commit details.

Code references:
- `/Users/emilzimmermann/gitviz/src/util/fmt.rs:42`
- `/Users/emilzimmermann/gitviz/src/util/fmt.rs:73`
- `/Users/emilzimmermann/gitviz/src/ui/widgets.rs:125`

Details:
- `format_iso(ts: i64)` converts `ts` to `u64`, then `days_to_ymd` iterates year-by-year in a loop.
- With very large timestamps (for example, `9223372036854775807`), this loop becomes effectively unbounded in practical runtime.
- The function is called during details rendering each frame, so the UI can stall before drawing.

Reproduction evidence:
- A crafted commit object in `/tmp/gitviz-negts` with `%at=9223372036854775807` was created.
- Running `cargo run -- --repo /tmp/gitviz-negts --max 10` in PTY enters alternate screen but does not complete first frame render.

Recommendation:
- Replace manual calendar conversion with a constant-time datetime conversion library (`time` or `chrono`) with explicit bounds checking.
- Reject or clamp unsupported timestamps before formatting.

## [MEDIUM] F-002: Silent commit dropping on malformed git-log records

Impact: Malformed commit metadata can be silently omitted from view, causing integrity gaps in what the user sees.

Code references:
- `/Users/emilzimmermann/gitviz/src/git/parser.rs:6`
- `/Users/emilzimmermann/gitviz/src/git/parser.rs:39`
- `/Users/emilzimmermann/gitviz/src/git/mod.rs:44`

Details:
- `parse_commits` uses `filter_map(parse_commit_record)`, so any parse failure drops the record without error.
- Timestamp parse failure (`parts[4].parse::<i64>().ok()?`) causes commit omission.
- `load_commits` does not surface dropped-record counts; malformed output can yield incomplete graphs silently.

Recommendation:
- Return structured parse diagnostics (parsed commits + rejected count/errors).
- Fail fast (or display warning) when non-empty output produces parse failures.

## Notes

- `run_git` uses `Command::new("git")` with direct argument passing (no shell invocation), which is good and avoids shell injection risk.
- Terminal control chars in displayed strings are mitigated by ratatui’s control-character filtering in buffer rendering.
