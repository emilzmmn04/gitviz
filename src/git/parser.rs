use super::model::Commit;

#[derive(Debug, Default)]
pub struct ParseCommitsReport {
    pub commits: Vec<Commit>,
    pub total_records: usize,
    pub rejected_records: usize,
    pub first_error: Option<String>,
}

/// Parse commits from `git log --format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%s%x1e` output.
/// Records are delimited by ASCII record separator (0x1e).
/// Fields within each record are delimited by ASCII unit separator (0x1f).
pub fn parse_commits(output: &str) -> ParseCommitsReport {
    let mut report = ParseCommitsReport::default();

    for record in output.split('\x1e') {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }

        report.total_records += 1;
        match parse_commit_record(record) {
            Ok(commit) => report.commits.push(commit),
            Err(err) => {
                report.rejected_records += 1;
                if report.first_error.is_none() {
                    report.first_error = Some(format!("record #{}: {}", report.total_records, err));
                }
            }
        }
    }

    report
}

fn parse_commit_record(record: &str) -> Result<Commit, String> {
    // splitn(6, ...) so that subject (field 6) is kept intact even if it
    // somehow contained the separator (unlikely but safe).
    let parts: Vec<&str> = record.splitn(6, '\x1f').collect();
    if parts.len() < 6 {
        return Err(format!("missing fields: expected 6, got {}", parts.len()));
    }

    let oid = parts[0].trim().to_string();
    if oid.is_empty() {
        return Err("empty commit hash".to_string());
    }

    let parents: Vec<String> = parts[1]
        .split_whitespace()
        .map(str::to_string)
        .filter(|s| !s.is_empty())
        .collect();

    let author = parts[2].trim().to_string();
    let author_email = parts[3].trim().to_string();
    let timestamp_raw = parts[4].trim();
    let timestamp = timestamp_raw
        .parse::<i64>()
        .map_err(|_| format!("invalid timestamp '{}'", timestamp_raw))?;
    // subject may have trailing \n from git's tformat
    let subject = parts[5].trim_end_matches('\n').to_string();

    Ok(Commit {
        oid,
        parents,
        author,
        author_email,
        timestamp,
        subject,
    })
}

/// Parse `git show-ref` output: `<oid> <refname>` per line.
/// Returns `Vec<(refname, oid)>`.
pub fn parse_show_ref(output: &str) -> Vec<(String, String)> {
    output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let mut parts = line.splitn(2, ' ');
            let oid = parts.next()?.trim().to_string();
            let refname = parts.next()?.trim().to_string();
            if oid.is_empty() || refname.is_empty() {
                None
            } else {
                Some((refname, oid))
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_show_ref_basic() {
        let input = "abc123def456abc123def456abc1 refs/heads/main\n\
                     def456abc789def456abc789def4 refs/heads/feature\n";
        let result = parse_show_ref(input);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0],
            (
                "refs/heads/main".to_string(),
                "abc123def456abc123def456abc1".to_string()
            )
        );
        assert_eq!(
            result[1],
            (
                "refs/heads/feature".to_string(),
                "def456abc789def456abc789def4".to_string()
            )
        );
    }

    #[test]
    fn test_parse_show_ref_empty() {
        let result = parse_show_ref("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_commit_record_with_parents() {
        let record = "abc123def456abc123def456abc1\x1f\
                      par1par1par1par1par1par1par1p par2par2par2par2par2par2par2p\x1f\
                      John Doe\x1f\
                      john@example.com\x1f\
                      1700000000\x1f\
                      Fix something important";
        let commit = parse_commit_record(record).expect("should parse");
        assert_eq!(commit.oid, "abc123def456abc123def456abc1");
        assert_eq!(commit.parents.len(), 2);
        assert_eq!(commit.author, "John Doe");
        assert_eq!(commit.author_email, "john@example.com");
        assert_eq!(commit.timestamp, 1700000000);
        assert_eq!(commit.subject, "Fix something important");
    }

    #[test]
    fn test_parse_commit_record_no_parents() {
        let record = "abc123def456abc123def456abc1\x1f\x1fJane Doe\x1fjane@example.com\x1f1700000001\x1fInitial commit";
        let commit = parse_commit_record(record).expect("should parse");
        assert_eq!(commit.oid, "abc123def456abc123def456abc1");
        assert!(commit.parents.is_empty());
        assert_eq!(commit.subject, "Initial commit");
    }

    #[test]
    fn test_parse_commits_multiple() {
        let record1 = "aaa\x1f\x1fAuth1\x1fa@b.com\x1f1000\x1fFirst";
        let record2 = "bbb\x1faaa\x1fAuth2\x1fb@c.com\x1f999\x1fSecond";
        let input = format!("{}\x1e{}\x1e", record1, record2);
        let report = parse_commits(&input);
        assert_eq!(report.total_records, 2);
        assert_eq!(report.rejected_records, 0);
        assert!(report.first_error.is_none());
        assert_eq!(report.commits.len(), 2);
        assert_eq!(report.commits[0].oid, "aaa");
        assert_eq!(report.commits[1].oid, "bbb");
        assert_eq!(report.commits[1].parents, vec!["aaa"]);
    }

    #[test]
    fn test_parse_commits_invalid_timestamp_is_rejected() {
        let bad = "bbb\x1faaa\x1fAuth2\x1fb@c.com\x1fnot-a-number\x1fSecond";
        let report = parse_commits(&format!("{}\x1e", bad));
        assert_eq!(report.total_records, 1);
        assert_eq!(report.rejected_records, 1);
        assert!(report.commits.is_empty());
        assert!(
            report
                .first_error
                .as_deref()
                .unwrap_or("")
                .contains("invalid timestamp")
        );
    }

    #[test]
    fn test_parse_commits_mixed_valid_and_invalid() {
        let good = "aaa\x1f\x1fAuth1\x1fa@b.com\x1f1000\x1fFirst";
        let bad = "bbb\x1faaa\x1fAuth2\x1fb@c.com\x1fnot-a-number\x1fSecond";
        let report = parse_commits(&format!("{}\x1e{}\x1e", good, bad));
        assert_eq!(report.total_records, 2);
        assert_eq!(report.rejected_records, 1);
        assert_eq!(report.commits.len(), 1);
        assert_eq!(report.commits[0].oid, "aaa");
    }
}
