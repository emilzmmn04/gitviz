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
    exclude_reachable_from: Option<&str>,
) -> Result<Vec<Commit>> {
    // Format: hash \x1f parents \x1f author \x1f email \x1f timestamp \x1f subject \x1f body \x1e
    // %x1f = ASCII unit separator (0x1f), %x1e = ASCII record separator (0x1e)
    let format_str = "--format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%s%x1f%b%x1e";
    let max_str = max.to_string();
    let max_count = format!("--max-count={}", max_str);

    let mut args: Vec<&str> = vec!["log", "--topo-order", format_str, &max_count];

    if all {
        args.push("--all");
    }

    // Exclude commits reachable from the given revision boundary.
    let not_arg: String;
    if let Some(s) = exclude_reachable_from {
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

/// Load refs (HEAD, local branches, remote branches, tags) from the repository.
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

    let ref_out = commands::try_run_git(repo, &["show-ref"]).unwrap_or_default();
    for (refname, oid) in parser::parse_show_ref(&ref_out) {
        if refname.starts_with("refs/heads/") {
            refs.branches.push((refname, oid));
        } else if refname.starts_with("refs/remotes/") {
            refs.remotes.push((refname, oid));
        } else if refname.starts_with("refs/tags/") {
            refs.tags.push((refname, oid));
        }
    }

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
    use super::{load_commits, load_refs, parse_git_log_output};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempRepo {
        path: PathBuf,
    }

    impl TempRepo {
        fn new() -> Self {
            let mut path = std::env::temp_dir();
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos();
            path.push(format!("gitviz-test-{}-{}", std::process::id(), unique));
            fs::create_dir_all(&path).expect("failed to create temp repo dir");

            run_git(&path, &["init", "-b", "main"]);
            run_git(&path, &["config", "user.name", "Gitviz Test"]);
            run_git(&path, &["config", "user.email", "gitviz@example.com"]);

            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn commit_file(&self, name: &str, contents: &str, message: &str) -> String {
            fs::write(self.path.join(name), contents).expect("failed to write test file");
            run_git(self.path(), &["add", name]);
            run_git(self.path(), &["commit", "-m", message]);
            run_git(self.path(), &["rev-parse", "HEAD"]).trim().to_string()
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn run_git(repo: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .expect("failed to run git command");

        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );

        String::from_utf8(output.stdout).expect("git output was not valid UTF-8")
    }

    #[test]
    fn test_parse_git_log_output_empty_is_ok() {
        let commits = parse_git_log_output("").expect("empty output should be accepted");
        assert!(commits.is_empty());
    }

    #[test]
    fn test_parse_git_log_output_valid_is_ok() {
        let output = "aaa\x1f\x1fAuth1\x1fa@b.com\x1f1000\x1fFirst\x1fBody\x1e";
        let commits = parse_git_log_output(output).expect("valid output should parse");
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].oid, "aaa");
        assert_eq!(commits[0].body, "Body");
    }

    #[test]
    fn test_parse_git_log_output_rejects_malformed_records() {
        let valid = "aaa\x1f\x1fAuth1\x1fa@b.com\x1f1000\x1fFirst\x1fBody";
        let invalid = "bbb\x1faaa\x1fAuth2\x1fb@c.com\x1fnot-a-number\x1fSecond\x1fBody";
        let output = format!("{}\x1e{}\x1e", valid, invalid);
        let err = parse_git_log_output(&output).expect_err("must fail when any row is malformed");
        let msg = err.to_string();
        assert!(msg.contains("rejected 1 of 2"));
        assert!(msg.contains("invalid timestamp"));
    }

    #[test]
    fn test_load_refs_includes_remote_branches() {
        let repo = TempRepo::new();
        let head = repo.commit_file("README.md", "hello\n", "initial commit");
        run_git(repo.path(), &["update-ref", "refs/remotes/origin/main", &head]);

        let refs = load_refs(repo.path()).expect("refs should load");
        assert!(
            refs.remotes
                .iter()
                .any(|(refname, oid)| refname == "refs/remotes/origin/main" && oid == &head)
        );
        assert!(refs.labels_for(&head).iter().any(|label| label == "origin/main"));
    }

    #[test]
    fn test_load_commits_excludes_revision_boundary() {
        let repo = TempRepo::new();
        let first = repo.commit_file("app.txt", "one\n", "first");
        let second = repo.commit_file("app.txt", "two\n", "second");

        let commits =
            load_commits(repo.path(), 50, true, Some(&first)).expect("commits should load");

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].oid, second);
    }
}
