#[derive(Debug, Clone)]
pub struct Commit {
    pub oid: String,
    pub parents: Vec<String>,
    pub author: String,
    pub author_email: String,
    pub timestamp: i64,
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChanged,
    Unmerged,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangedFile {
    pub path: String,
    pub change_kind: ChangeKind,
    pub old_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CommitInspectData {
    pub changed_files: Vec<ChangedFile>,
    pub file_list_truncated: bool,
    pub diff_text: String,
    pub diff_truncated: bool,
}

#[derive(Debug, Clone)]
pub enum InspectCacheEntry {
    Loading,
    Ready(CommitInspectData),
    Error(String),
}

#[derive(Debug, Default, Clone)]
pub struct Refs {
    pub head_oid: String,
    pub head_name: Option<String>, // "refs/heads/main" or None for detached
    pub branches: Vec<(String, String)>, // (refname, oid)
    pub remotes: Vec<(String, String)>,  // (refname, oid)
    pub tags: Vec<(String, String)>,     // (refname, oid)
}

impl Refs {
    /// Returns a short label for a given OID, listing HEAD, branch names, tags.
    pub fn labels_for(&self, oid: &str) -> Vec<String> {
        let mut labels = Vec::new();

        // HEAD indicator
        if self.head_oid == oid {
            if let Some(ref name) = self.head_name {
                // e.g. "refs/heads/main" → "HEAD -> main"
                let short = name
                    .strip_prefix("refs/heads/")
                    .unwrap_or(name.as_str());
                push_unique(&mut labels, format!("HEAD -> {}", short));
            } else {
                push_unique(&mut labels, "HEAD".to_string());
            }
        }

        // Branch labels
        for (refname, ref_oid) in &self.branches {
            if ref_oid == oid {
                let short = refname
                    .strip_prefix("refs/heads/")
                    .unwrap_or(refname.as_str());
                // Skip if already covered by HEAD label
                if self.head_name.as_deref() != Some(refname.as_str()) || self.head_oid != oid {
                    push_unique(&mut labels, short.to_string());
                }
            }
        }

        // Remote branch labels
        for (refname, ref_oid) in &self.remotes {
            if ref_oid == oid {
                let short = refname
                    .strip_prefix("refs/remotes/")
                    .unwrap_or(refname.as_str());
                push_unique(&mut labels, short.to_string());
            }
        }

        // Tag labels
        for (refname, ref_oid) in &self.tags {
            if ref_oid == oid {
                let short = refname
                    .strip_prefix("refs/tags/")
                    .unwrap_or(refname.as_str());
                push_unique(&mut labels, format!("tag: {}", short));
            }
        }

        labels
    }
}

fn push_unique(labels: &mut Vec<String>, label: String) {
    if !labels.iter().any(|existing| existing == &label) {
        labels.push(label);
    }
}
