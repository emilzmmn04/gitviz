pub mod commands;
pub mod model;
pub mod parser;

use anyhow::{bail, Context, Result};
use model::{Commit, CommitInspectData, Refs};
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
    let mut refs = Refs {
        // HEAD oid
        head_oid: commands::run_git(repo, &["rev-parse", "HEAD"])
            .unwrap_or_default()
            .trim()
            .to_string(),
        // Symbolic HEAD (branch name) — exits non-zero when detached
        head_name: commands::try_run_git(repo, &["symbolic-ref", "-q", "HEAD"])
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        ..Refs::default()
    };

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

pub fn load_commit_inspect_data(repo: &Path, oid: &str) -> Result<CommitInspectData> {
    let files_output = commands::run_git(
        repo,
        &["show", "--format=", "--name-status", "--find-renames", "--find-copies", oid],
    )
    .with_context(|| format!("Failed to load changed files for commit {}", oid))?;
    let mut changed_files = parser::parse_changed_files(&files_output);
    let mut file_list_truncated = false;
    if changed_files.len() > 1000 {
        changed_files.truncate(1000);
        file_list_truncated = true;
    }

    let diff_output = commands::run_git(
        repo,
        &[
            "show",
            "--format=medium",
            "--find-renames",
            "--find-copies",
            "--patch",
            oid,
        ],
    )
    .with_context(|| format!("Failed to load diff for commit {}", oid))?;

    let (mut diff_text, diff_truncated) = truncate_diff_preview(&diff_output, 400);
    if diff_text.trim().is_empty() {
        diff_text = "(no patch content)".to_string();
    }

    Ok(CommitInspectData {
        changed_files,
        file_list_truncated,
        diff_text,
        diff_truncated,
    }
    )
}

pub fn github_commit_url(repo: &Path, oid: &str) -> Option<String> {
    let remote_url = commands::try_run_git(repo, &["config", "--get", "remote.origin.url"])?;
    let (owner, repo_name) = parse_github_remote_url(remote_url.trim())?;
    Some(format!(
        "https://github.com/{owner}/{repo_name}/commit/{oid}"
    ))
}

fn truncate_diff_preview(diff_output: &str, max_lines: usize) -> (String, bool) {
    let mut lines: Vec<&str> = diff_output.lines().collect();
    let truncated = lines.len() > max_lines;
    if truncated {
        lines.truncate(max_lines);
    }

    let mut text = lines.join("\n");
    if truncated {
        if !text.is_empty() {
            text.push('\n');
        }
        text.push_str("... diff truncated; open in GitHub or use git show for the full patch");
    }

    (text, truncated)
}

fn parse_github_remote_url(remote: &str) -> Option<(String, String)> {
    let remote = remote.trim();

    let stripped = if let Some(rest) = remote.strip_prefix("git@github.com:") {
        rest
    } else if let Some(rest) = remote.strip_prefix("ssh://git@github.com/") {
        rest
    } else if let Some(rest) = remote.strip_prefix("https://github.com/") {
        rest
    } else {
        return None;
    };

    let stripped = stripped.strip_suffix(".git").unwrap_or(stripped);
    let mut parts = stripped.split('/');
    let owner = parts.next()?.trim();
    let repo_name = parts.next()?.trim();
    if owner.is_empty() || repo_name.is_empty() || parts.next().is_some() {
        return None;
    }

    Some((owner.to_string(), repo_name.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        github_commit_url, load_commit_inspect_data, load_commits, load_refs,
        parse_git_log_output, parse_github_remote_url, truncate_diff_preview,
    };
    use crate::git::model::ChangeKind;
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

        fn commit_file_with_body(
            &self,
            name: &str,
            contents: &str,
            subject: &str,
            body: &str,
        ) -> String {
            fs::write(self.path.join(name), contents).expect("failed to write test file");
            run_git(self.path(), &["add", name]);
            run_git(self.path(), &["commit", "-m", subject, "-m", body]);
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

    #[test]
    fn test_load_commit_inspect_data_reads_files_and_diff() {
        let repo = TempRepo::new();
        let oid = repo.commit_file_with_body("app.txt", "one\n", "subject", "body");
        let data = load_commit_inspect_data(repo.path(), &oid).expect("inspect data should load");
        assert_eq!(data.changed_files.len(), 1);
        assert_eq!(data.changed_files[0].change_kind, ChangeKind::Added);
        assert!(data.diff_text.contains("subject"));
        assert!(!data.diff_truncated);
        assert!(!data.file_list_truncated);
    }

    #[test]
    fn test_parse_github_remote_url_variants() {
        let ssh = parse_github_remote_url("git@github.com:owner/repo.git");
        let ssh_scheme = parse_github_remote_url("ssh://git@github.com/owner/repo.git");
        let https = parse_github_remote_url("https://github.com/owner/repo.git");
        let https_plain = parse_github_remote_url("https://github.com/owner/repo");
        assert_eq!(ssh, Some(("owner".to_string(), "repo".to_string())));
        assert_eq!(ssh_scheme, Some(("owner".to_string(), "repo".to_string())));
        assert_eq!(https, Some(("owner".to_string(), "repo".to_string())));
        assert_eq!(https_plain, Some(("owner".to_string(), "repo".to_string())));
        assert!(parse_github_remote_url("git@gitlab.com:owner/repo.git").is_none());
    }

    #[test]
    fn test_truncate_diff_preview_limits_lines() {
        let diff = (0..450)
            .map(|i| format!("line-{i}"))
            .collect::<Vec<_>>()
            .join("\n");
        let (text, truncated) = truncate_diff_preview(&diff, 400);
        assert!(truncated);
        assert!(text.contains("line-399"));
        assert!(text.contains("... diff truncated; open in GitHub or use git show for the full patch"));
    }

    #[test]
    fn test_github_commit_url_uses_origin_remote() {
        let repo = TempRepo::new();
        let oid = repo.commit_file("README.md", "hello\n", "initial commit");
        run_git(
            repo.path(),
            &["remote", "add", "origin", "https://github.com/owner/repo.git"],
        );
        let url = github_commit_url(repo.path(), &oid).expect("github url should resolve");
        assert_eq!(url, format!("https://github.com/owner/repo/commit/{oid}"));
    }
}
