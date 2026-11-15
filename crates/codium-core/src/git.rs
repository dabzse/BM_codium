//! # OmniGit Engine & Git CLI Integration
//!
//! High-performance Git engine for `BM_codium`, integrated from `dabzse/omnigit`.
//! Provides Git status tracking, staging/unstaging, commit log parsing with
//! ASCII unit delimiters, branch management, diffs, and inline blame.

use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

/// ASCII Record and Unit separators matching `@dabzse/omnigit-core`
pub const FIELD_SEP: char = '\x1e';
pub const COMMIT_SEP: char = '\x1f';

/// Status of a file in the working tree or index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileChangeStatus {
    Modified,
    Added,
    Deleted,
    Renamed,
    Copied,
    Untracked,
    Conflicted,
}

impl FileChangeStatus {
    pub fn badge(&self) -> &'static str {
        match self {
            Self::Modified => "M",
            Self::Added => "A",
            Self::Deleted => "D",
            Self::Renamed => "R",
            Self::Copied => "C",
            Self::Untracked => "U",
            Self::Conflicted => "!",
        }
    }

    pub fn color_hex(&self) -> &'static str {
        match self {
            Self::Modified => "#e5c07b",   // Yellow / warning
            Self::Added => "#98c379",      // Green
            Self::Deleted => "#e06c75",    // Red
            Self::Renamed => "#61afef",    // Blue
            Self::Copied => "#56b6c2",     // Cyan
            Self::Untracked => "#73c991",  // Light green
            Self::Conflicted => "#ff5370", // Bright red
        }
    }
}

/// A changed file in git
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitFileChange {
    pub path: String,
    pub status: FileChangeStatus,
    pub is_staged: bool,
    pub old_path: Option<String>,
}

/// Git branch metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
    pub is_remote: bool,
    pub upstream: Option<String>,
}

/// A parsed Git commit matching `@dabzse/omnigit-core`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub short_hash: String,
    pub parents: Vec<String>,
    pub subject: String,
    pub body: String,
    pub author_name: String,
    pub author_email: String,
    pub author_date: i64,
    pub refs: Vec<String>,
}

/// Comprehensive repository status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GitRepoStatus {
    pub is_repo: bool,
    pub root_dir: Option<PathBuf>,
    pub current_branch: String,
    pub ahead: usize,
    pub behind: usize,
    pub staged: Vec<GitFileChange>,
    pub unstaged: Vec<GitFileChange>,
    pub untracked: Vec<GitFileChange>,
}

impl GitRepoStatus {
    pub fn total_changes(&self) -> usize {
        self.staged.len() + self.unstaged.len() + self.untracked.len()
    }

    pub fn has_changes(&self) -> bool {
        self.total_changes() > 0
    }
}

/// Inline Git blame information for a line
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitBlameLine {
    pub hash: String,
    pub short_hash: String,
    pub author: String,
    pub date: String,
    pub summary: String,
    pub line_number: usize,
}

/// Low-level executor wrapping the system `git` CLI with isolation settings
pub struct GitExecutor;

impl GitExecutor {
    /// Execute a git command and return stdout
    pub fn exec(cwd: &Path, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .env("LC_ALL", "C")
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .with_context(|| format!("Failed to spawn git with args: {:?}", args))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Err(anyhow!(
                "git {} failed (exit code {}): {}",
                args.join(" "),
                output.status.code().unwrap_or(-1),
                stderr.trim()
            ))
        }
    }

    /// Try executing a git command, returning None on failure
    pub fn try_exec(cwd: &Path, args: &[&str]) -> Option<String> {
        Self::exec(cwd, args).ok()
    }

    /// Check if the directory is inside a git repository
    pub fn is_git_repo(cwd: &Path) -> bool {
        Self::try_exec(cwd, &["rev-parse", "--is-inside-work-tree"])
            .map(|out| out.trim() == "true")
            .unwrap_or(false)
    }

    /// Get root path of the git repository
    pub fn get_repo_root(cwd: &Path) -> Option<PathBuf> {
        Self::try_exec(cwd, &["rev-parse", "--show-toplevel"])
            .map(|s| PathBuf::from(s.trim()))
    }

    /// Get the current repository status (branches, staged, unstaged, untracked)
    pub fn get_status(cwd: &Path) -> Result<GitRepoStatus> {
        if !Self::is_git_repo(cwd) {
            return Ok(GitRepoStatus {
                is_repo: false,
                ..Default::default()
            });
        }

        let root_dir = Self::get_repo_root(cwd);
        let raw = Self::exec(cwd, &["status", "--porcelain=v1", "-b"])?;
        let status = Self::parse_status(&raw, root_dir)?;
        Ok(status)
    }

    /// Parse `git status --porcelain=v1 -b` output
    pub fn parse_status(raw: &str, root_dir: Option<PathBuf>) -> Result<GitRepoStatus> {
        let mut status = GitRepoStatus {
            is_repo: true,
            root_dir,
            current_branch: "master".to_string(),
            ahead: 0,
            behind: 0,
            staged: Vec::new(),
            unstaged: Vec::new(),
            untracked: Vec::new(),
        };

        for line in raw.lines() {
            if line.starts_with("## ") {
                // Branch header: ## master...origin/master [ahead 1, behind 2]
                let branch_info = line[3..].trim();
                Self::parse_branch_header(branch_info, &mut status);
            } else if line.len() >= 3 {
                let index_status = line.chars().nth(0).unwrap_or(' ');
                let work_status = line.chars().nth(1).unwrap_or(' ');
                let path_part = line[3..].trim();

                // Handle renamed files: old_path -> new_path, stripping any git quotes
                let (path, old_path) = if let Some((old_p, new_p)) = path_part.split_once(" -> ") {
                    (
                        new_p.trim().trim_matches('"').to_string(),
                        Some(old_p.trim().trim_matches('"').to_string()),
                    )
                } else {
                    (path_part.trim().trim_matches('"').to_string(), None)
                };

                // Untracked files
                if index_status == '?' && work_status == '?' {
                    status.untracked.push(GitFileChange {
                        path,
                        status: FileChangeStatus::Untracked,
                        is_staged: false,
                        old_path: None,
                    });
                    continue;
                }

                // Staged changes (Index status is not space and not '?')
                if index_status != ' ' && index_status != '?' {
                    let st = match index_status {
                        'M' => FileChangeStatus::Modified,
                        'A' => FileChangeStatus::Added,
                        'D' => FileChangeStatus::Deleted,
                        'R' => FileChangeStatus::Renamed,
                        'C' => FileChangeStatus::Copied,
                        'U' => FileChangeStatus::Conflicted,
                        _ => FileChangeStatus::Modified,
                    };
                    status.staged.push(GitFileChange {
                        path: path.clone(),
                        status: st,
                        is_staged: true,
                        old_path: old_path.clone(),
                    });
                }

                // Unstaged changes (Working tree status is not space and not '?')
                if work_status != ' ' && work_status != '?' {
                    let st = match work_status {
                        'M' => FileChangeStatus::Modified,
                        'D' => FileChangeStatus::Deleted,
                        'U' => FileChangeStatus::Conflicted,
                        _ => FileChangeStatus::Modified,
                    };
                    status.unstaged.push(GitFileChange {
                        path,
                        status: st,
                        is_staged: false,
                        old_path,
                    });
                }
            }
        }

        Ok(status)
    }

    fn parse_branch_header(header: &str, status: &mut GitRepoStatus) {
        // Examples:
        // master...origin/master [ahead 5]
        // master...origin/master [behind 1]
        // master...origin/master [ahead 2, behind 3]
        // master (no upstream)
        // HEAD (no branch)
        let (branch_part, meta_part) = if let Some((b, m)) = header.split_once(" [") {
            (b.trim(), Some(m.trim_end_matches(']').trim()))
        } else {
            (header.trim(), None)
        };

        let branch_name = if let Some((local, _)) = branch_part.split_once("...") {
            local.trim()
        } else {
            branch_part
        };

        status.current_branch = branch_name.to_string();

        if let Some(meta) = meta_part {
            for part in meta.split(',') {
                let trimmed = part.trim();
                if let Some(count_str) = trimmed.strip_prefix("ahead ") {
                    status.ahead = count_str.parse().unwrap_or(0);
                } else if let Some(count_str) = trimmed.strip_prefix("behind ") {
                    status.behind = count_str.parse().unwrap_or(0);
                }
            }
        }
    }

    /// List all branches in the repository
    pub fn get_branches(cwd: &Path) -> Result<Vec<GitBranch>> {
        let raw = Self::exec(
            cwd,
            &[
                "branch",
                "-a",
                "--format=%(refname:short)\x1e%(HEAD)\x1e%(upstream:short)",
            ],
        )?;

        let mut branches = Vec::new();
        for line in raw.lines() {
            let parts: Vec<&str> = line.split(FIELD_SEP).collect();
            if parts.is_empty() {
                continue;
            }
            let name = parts[0].trim().to_string();
            if name.is_empty() {
                continue;
            }
            let is_current = parts.get(1).map(|&h| h.trim() == "*").unwrap_or(false);
            let upstream = parts
                .get(2)
                .map(|&u| u.trim().to_string())
                .filter(|u| !u.is_empty());
            let is_remote = name.starts_with("origin/") || name.contains('/');

            branches.push(GitBranch {
                name,
                is_current,
                is_remote,
                upstream,
            });
        }
        Ok(branches)
    }

    /// Fetch git commit log using OmniGit's robust ASCII unit separator format
    pub fn get_log(cwd: &Path, max_count: usize) -> Result<Vec<GitCommit>> {
        let count_str = max_count.to_string();
        let format_str = format!(
            "%H{f}%h{f}%P{f}%s{f}%b{f}%an{f}%ae{f}%at{f}%D{c}",
            f = FIELD_SEP,
            c = COMMIT_SEP
        );

        let raw = Self::exec(
            cwd,
            &["log", &format!("--format={}", format_str), "-n", &count_str],
        )?;

        Self::parse_log(&raw)
    }

    /// Parse git log raw string
    pub fn parse_log(raw: &str) -> Result<Vec<GitCommit>> {
        let mut commits = Vec::new();
        for entry in raw.split(COMMIT_SEP) {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }

            let fields: Vec<&str> = entry.split(FIELD_SEP).collect();
            if fields.len() < 9 {
                continue;
            }

            let hash = fields[0].trim().to_string();
            let short_hash = fields[1].trim().to_string();
            let parents = fields[2]
                .trim()
                .split_whitespace()
                .map(String::from)
                .collect();
            let subject = fields[3].trim().to_string();
            let body = fields[4].trim().to_string();
            let author_name = fields[5].trim().to_string();
            let author_email = fields[6].trim().to_string();
            let author_date = fields[7].trim().parse::<i64>().unwrap_or(0);
            let refs = fields[8]
                .split(',')
                .map(|r| r.trim().to_string())
                .filter(|r| !r.is_empty())
                .collect();

            commits.push(GitCommit {
                hash,
                short_hash,
                parents,
                subject,
                body,
                author_name,
                author_email,
                author_date,
                refs,
            });
        }
        Ok(commits)
    }

    /// Stage a file (`git add -- <file>`)
    pub fn stage_file(cwd: &Path, file_path: &str) -> Result<()> {
        let clean = file_path.trim().trim_matches('"');
        Self::exec(cwd, &["add", "--", clean])?;
        Ok(())
    }

    /// Unstage a file (`git restore --staged -- <file>`)
    pub fn unstage_file(cwd: &Path, file_path: &str) -> Result<()> {
        let clean = file_path.trim().trim_matches('"');
        // Try git restore --staged first, fallback to git reset HEAD
        if Self::exec(cwd, &["restore", "--staged", "--", clean]).is_err() {
            Self::exec(cwd, &["reset", "HEAD", "--", clean])?;
        }
        Ok(())
    }

    /// Stage all modified and untracked files (`git add -A`)
    pub fn stage_all(cwd: &Path) -> Result<()> {
        Self::exec(cwd, &["add", "-A"])?;
        Ok(())
    }

    /// Unstage all files (`git reset HEAD`)
    pub fn unstage_all(cwd: &Path) -> Result<()> {
        Self::exec(cwd, &["reset", "HEAD"])?;
        Ok(())
    }

    /// Commit staged changes with message (`git commit -m <message>`)
    pub fn commit(cwd: &Path, message: &str) -> Result<String> {
        let msg = message.trim();
        if msg.is_empty() {
            return Err(anyhow!("Commit message cannot be empty"));
        }
        Self::exec(cwd, &["commit", "-m", msg])
    }

    /// Get unified diff for a file or entire working directory
    pub fn get_diff(cwd: &Path, file_path: Option<&str>, staged: bool) -> Result<String> {
        let mut args = vec!["diff"];
        if staged {
            args.push("--staged");
        }
        if let Some(f) = file_path {
            args.push("--");
            args.push(f);
        }
        Self::exec(cwd, &args)
    }

    /// Get line blame information
    pub fn get_blame_for_line(
        cwd: &Path,
        file_path: &str,
        line: usize,
    ) -> Result<Option<GitBlameLine>> {
        let line_range = format!("{},{}", line, line);
        let raw = match Self::exec(
            cwd,
            &["blame", "-L", &line_range, "--porcelain", "--", file_path],
        ) {
            Ok(out) => out,
            Err(_) => return Ok(None),
        };

        // Porcelain blame format:
        // <hash> <orig_line> <final_line> [<num_lines>]
        // author <name>
        // author-mail <email>
        // author-time <unix>
        // summary <message>
        let mut hash = String::new();
        let mut author = String::new();
        let mut summary = String::new();
        let mut date = String::new();

        for l in raw.lines() {
            if hash.is_empty() {
                if let Some(first_word) = l.split_whitespace().next() {
                    hash = first_word.to_string();
                }
            } else if let Some(a) = l.strip_prefix("author ") {
                author = a.trim().to_string();
            } else if let Some(s) = l.strip_prefix("summary ") {
                summary = s.trim().to_string();
            } else if let Some(t) = l.strip_prefix("author-time ") {
                if let Ok(ts) = t.trim().parse::<i64>() {
                    // Simple relative or formatted date
                    date = format_timestamp_relative(ts);
                }
            }
        }

        if hash.is_empty() {
            return Ok(None);
        }

        let short_hash = if hash.len() >= 7 {
            hash[..7].to_string()
        } else {
            hash.clone()
        };

        Ok(Some(GitBlameLine {
            hash,
            short_hash,
            author,
            date,
            summary,
            line_number: line,
        }))
    }

    /// Checkout an existing branch
    pub fn checkout_branch(cwd: &Path, branch_name: &str) -> Result<()> {
        Self::exec(cwd, &["checkout", branch_name])?;
        Ok(())
    }

    /// Create and checkout a new branch
    pub fn create_branch(cwd: &Path, branch_name: &str) -> Result<()> {
        Self::exec(cwd, &["checkout", "-b", branch_name])?;
        Ok(())
    }
}

/// Helper function to format unix timestamp into friendly relative string
pub fn format_timestamp_relative(timestamp: i64) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let diff = now - timestamp;
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{} min ago", diff / 60)
    } else if diff < 86400 {
        format!("{} hours ago", diff / 3600)
    } else if diff < 604800 {
        format!("{} days ago", diff / 86400)
    } else {
        format!("{} weeks ago", diff / 604800)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status() {
        let raw = r#"## master...origin/master [ahead 2, behind 1]
M  crates/codium-core/src/lib.rs
 M crates/codium-ui/src/views/activity_bar.rs
?? settings/keymaps/vscode.json
"#;
        let status = GitExecutor::parse_status(raw, None).unwrap();
        assert_eq!(status.current_branch, "master");
        assert_eq!(status.ahead, 2);
        assert_eq!(status.behind, 1);
        assert_eq!(status.staged.len(), 1);
        assert_eq!(status.staged[0].path, "crates/codium-core/src/lib.rs");
        assert_eq!(status.staged[0].status, FileChangeStatus::Modified);
        assert!(status.staged[0].is_staged);

        assert_eq!(status.unstaged.len(), 1);
        assert_eq!(status.unstaged[0].path, "crates/codium-ui/src/views/activity_bar.rs");
        assert!(!status.unstaged[0].is_staged);

        assert_eq!(status.untracked.len(), 1);
        assert_eq!(status.untracked[0].path, "settings/keymaps/vscode.json");
    }

    #[test]
    fn test_parse_log() {
        let entry = format!(
            "d76feb123456{f}d76feb1{f}parent1 parent2{f}feat: my commit{f}commit body line{f}Mihaly Nyilas{f}dev@example.com{f}1700000000{f}HEAD -> master, origin/master{c}",
            f = FIELD_SEP,
            c = COMMIT_SEP
        );
        let commits = GitExecutor::parse_log(&entry).unwrap();
        assert_eq!(commits.len(), 1);
        let c = &commits[0];
        assert_eq!(c.short_hash, "d76feb1");
        assert_eq!(c.subject, "feat: my commit");
        assert_eq!(c.author_name, "Mihaly Nyilas");
        assert_eq!(c.parents.len(), 2);
        assert_eq!(c.refs.len(), 2);
    }

    #[test]
    fn test_current_repo_status() {
        let cwd = Path::new(".");
        if GitExecutor::is_git_repo(cwd) {
            let status = GitExecutor::get_status(cwd).unwrap();
            assert!(status.is_repo);
            assert!(!status.current_branch.is_empty());
        }
    }
}
