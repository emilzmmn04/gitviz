pub mod commands;
pub mod model;
pub mod parser;

use anyhow::{bail, Context, Result};
use model::{Commit, Refs};
use std::path::Path;

/// Load commits from the repository using a single `git log` call.
pub fn load_commits(
    repo: &Path,
    max: usize,
    all: bool,
    since: Option<&str>,
) -> Result<Vec<Commit>> {
    // Format: hash \x1f parents \x1f author \x1f email \x1f timestamp \x1f subject \x1e
    // %x1f = ASCII unit separator (0x1f), %x1e = ASCII record separator (0x1e)
    let format_str = "--format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%s%x1e";
    let max_str = max.to_string();
    let max_count = format!("--max-count={}", max_str);

    let mut args: Vec<&str> = vec!["log", "--topo-order", format_str, &max_count];

    if all {
        args.push("--all");
    }

    // --since acts as an exclusion boundary: show commits NOT reachable from <rev>
    let not_arg: String;
    if let Some(s) = since {
        not_arg = s.to_string();
        args.push("--not");
        args.push(&not_arg);
    }

    // If not --all, specify HEAD explicitly
    if !all {
        args.push("HEAD");
    }

    let output = commands::run_git(repo, &args)
        .with_context(|| format!("Failed to load commits from {}", repo.display()))?;

    parse_git_log_output(&output)
}

fn parse_git_log_output(output: &str) -> Result<Vec<Commit>> {
    let report = parser::parse_commits(output);
    if report.commits.is_empty() && output.trim().is_empty() {
        // Repository might be empty (no commits yet)
        return Ok(Vec::new());
    }

    if report.rejected_records > 0 {
        let first_error = report
            .first_error
            .as_deref()
            .unwrap_or("unknown parse error");
        bail!(
            "Malformed git log output: rejected {} of {} record(s); first error: {}",
            report.rejected_records,
            report.total_records,
            first_error
        );
    }

    Ok(report.commits)
}

/// Load refs (HEAD, branches, tags) from the repository.
pub fn load_refs(repo: &Path) -> Result<Refs> {
    let mut refs = Refs::default();

    // HEAD oid
    refs.head_oid = commands::run_git(repo, &["rev-parse", "HEAD"])
        .unwrap_or_default()
        .trim()
        .to_string();

    // Symbolic HEAD (branch name) — exits non-zero when detached
    refs.head_name = commands::try_run_git(repo, &["symbolic-ref", "-q", "HEAD"])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    // Branches
    let branch_out = commands::try_run_git(repo, &["show-ref", "--heads"]).unwrap_or_default();
    refs.branches = parser::parse_show_ref(&branch_out);

    // Tags
    let tag_out = commands::try_run_git(repo, &["show-ref", "--tags"]).unwrap_or_default();
    refs.tags = parser::parse_show_ref(&tag_out);

    Ok(refs)
}

/// Verify the path is inside a git repository.
pub fn check_repo(repo: &Path) -> Result<()> {
    commands::run_git(repo, &["rev-parse", "--git-dir"])
        .with_context(|| format!("{} is not a git repository", repo.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::parse_git_log_output;

    #[test]
    fn test_parse_git_log_output_empty_is_ok() {
        let commits = parse_git_log_output("").expect("empty output should be accepted");
        assert!(commits.is_empty());
    }

    #[test]
    fn test_parse_git_log_output_valid_is_ok() {
        let output = "aaa\x1f\x1fAuth1\x1fa@b.com\x1f1000\x1fFirst\x1e";
        let commits = parse_git_log_output(output).expect("valid output should parse");
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].oid, "aaa");
    }

    #[test]
    fn test_parse_git_log_output_rejects_malformed_records() {
        let valid = "aaa\x1f\x1fAuth1\x1fa@b.com\x1f1000\x1fFirst";
        let invalid = "bbb\x1faaa\x1fAuth2\x1fb@c.com\x1fnot-a-number\x1fSecond";
        let output = format!("{}\x1e{}\x1e", valid, invalid);
        let err = parse_git_log_output(&output).expect_err("must fail when any row is malformed");
        let msg = err.to_string();
        assert!(msg.contains("rejected 1 of 2"));
        assert!(msg.contains("invalid timestamp"));
    }
}
