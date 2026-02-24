use super::model::Commit;

/// Parse commits from `git log --format=%H%x1f%P%x1f%an%x1f%ae%x1f%at%x1f%s%x1e` output.
/// Records are delimited by ASCII record separator (0x1e).
/// Fields within each record are delimited by ASCII unit separator (0x1f).
pub fn parse_commits(output: &str) -> Vec<Commit> {
    output
        .split('\x1e')
        .filter_map(parse_commit_record)
        .collect()
}

fn parse_commit_record(record: &str) -> Option<Commit> {
    let record = record.trim();
    if record.is_empty() {
        return None;
    }

    // splitn(6, ...) so that subject (field 6) is kept intact even if it
    // somehow contained the separator (unlikely but safe).
    let parts: Vec<&str> = record.splitn(6, '\x1f').collect();
    if parts.len() < 6 {
        return None;
    }

    let oid = parts[0].trim().to_string();
    if oid.is_empty() {
        return None;
    }

    let parents: Vec<String> = parts[1]
        .split_whitespace()
        .map(str::to_string)
        .filter(|s| !s.is_empty())
        .collect();

    let author = parts[2].trim().to_string();
    let author_email = parts[3].trim().to_string();
    let timestamp = parts[4].trim().parse::<i64>().ok()?;
    // subject may have trailing \n from git's tformat
    let subject = parts[5].trim_end_matches('\n').to_string();

    Some(Commit {
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
        let commits = parse_commits(&input);
        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].oid, "aaa");
        assert_eq!(commits[1].oid, "bbb");
        assert_eq!(commits[1].parents, vec!["aaa"]);
    }
}
