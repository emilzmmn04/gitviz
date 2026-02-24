pub mod commands;
pub mod model;
pub mod parser;

use anyhow::{Context, Result};
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

    let commits = parser::parse_commits(&output);
    if commits.is_empty() && output.trim().is_empty() {
        // Repository might be empty (no commits yet)
        return Ok(Vec::new());
    }

    Ok(commits)
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
