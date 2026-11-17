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
    Staged,
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
            Self::Staged => "S",
        }
    }

    pub fn color_hex(&self) -> &'static str {
        match self {
            Self::Modified => "#e5c07b",   // Warm Amber / Yellow (Modified)
            Self::Added => "#98c379",      // Green (Added)
            Self::Deleted => "#e06c75",    // Red (Deleted)
            Self::Renamed => "#61afef",    // Blue (Renamed)
            Self::Copied => "#56b6c2",     // Cyan (Copied)
            Self::Untracked => "#73c991",  // Light Green (Untracked)
            Self::Conflicted => "#ff5370", // Bright Red (Conflicted)
            Self::Staged => "#61afef",     // Vibrant Blue / Azure (Staged)
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

/// Palette of distinct branch colors used for commit graph lanes
pub const GRAPH_LANE_COLORS: &[&str] = &[
    "#3b82f6", // Blue (primary)
    "#10b981", // Emerald Green
    "#f59e0b", // Amber
    "#a855f7", // Purple
    "#06b6d4", // Cyan
    "#ec4899", // Pink
    "#eab308", // Yellow
    "#14b8a6", // Teal
];

/// Layout information for a commit in the visual OmniGit commit graph
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitGraphNode {
    pub commit: GitCommit,
    pub lane: usize,
    pub total_lanes: usize,
    pub active_lanes: Vec<bool>,
    pub is_head: bool,
    pub is_merge: bool,
    pub is_root: bool,
}

impl GitGraphNode {
    /// Return the hex color string for this node's branch lane
    pub fn lane_color(&self) -> &'static str {
        GRAPH_LANE_COLORS[self.lane % GRAPH_LANE_COLORS.len()]
    }

    /// Return the hex color string for any given lane index
    pub fn color_for_lane(lane: usize) -> &'static str {
        GRAPH_LANE_COLORS[lane % GRAPH_LANE_COLORS.len()]
    }
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

    /// Lookup git status for a specific file path (relative to repo root)
    pub fn get_file_change(&self, rel_path: &str) -> Option<GitFileChange> {
        let clean = rel_path.trim_start_matches("./").trim_start_matches('/');
        // Check staged first (if staged, it takes precedence for visual S / A / D badge)
        if let Some(f) = self.staged.iter().find(|f| f.path == clean) {
            return Some(f.clone());
        }
        // Then unstaged
        if let Some(f) = self.unstaged.iter().find(|f| f.path == clean) {
            return Some(f.clone());
        }
        // Then untracked
        if let Some(f) = self.untracked.iter().find(|f| f.path == clean) {
            return Some(f.clone());
        }
        None
    }

    /// Robust lookup of git status for any file path (absolute or relative, canonicalized or raw)
    pub fn find_file_change(&self, file_path: &Path, repo_root: Option<&Path>) -> Option<GitFileChange> {
        if !self.is_repo {
            return None;
        }

        // 1. Try stripping repo_root or self.root_dir
        let root_candidates = [repo_root, self.root_dir.as_deref()];
        for root_opt in root_candidates.into_iter().flatten() {
            if let Ok(rel) = file_path.strip_prefix(root_opt) {
                if let Some(rel_str) = rel.to_str() {
                    if let Some(c) = self.get_file_change(rel_str) {
                        return Some(c);
                    }
                }
            }
            if let (Ok(c_file), Ok(c_root)) = (file_path.canonicalize(), root_opt.canonicalize()) {
                if let Ok(rel) = c_file.strip_prefix(&c_root) {
                    if let Some(rel_str) = rel.to_str() {
                        if let Some(c) = self.get_file_change(rel_str) {
                            return Some(c);
                        }
                    }
                }
            }
        }

        // 2. Direct string match
        if let Some(path_str) = file_path.to_str() {
            if let Some(c) = self.get_file_change(path_str) {
                return Some(c);
            }
        }

        // 3. Fallback: match by path suffix (ends_with)
        for f in &self.staged {
            if file_path.ends_with(&f.path) {
                return Some(f.clone());
            }
        }
        for f in &self.unstaged {
            if file_path.ends_with(&f.path) {
                return Some(f.clone());
            }
        }
        for f in &self.untracked {
            if file_path.ends_with(&f.path) {
                return Some(f.clone());
            }
        }

        None
    }

    /// Check if a directory contains any staged, unstaged, or untracked changes
    pub fn dir_has_changes(&self, dir_path: &Path, repo_root: Option<&Path>) -> Option<FileChangeStatus> {
        if !self.is_repo {
            return None;
        }

        let root_candidates = [repo_root, self.root_dir.as_deref()];
        let check_list = |list: &[GitFileChange]| -> bool {
            for f in list {
                let f_path = Path::new(&f.path);
                if f_path.starts_with(dir_path) {
                    return true;
                }
                for root in root_candidates.into_iter().flatten() {
                    let full_path = root.join(f_path);
                    if full_path.starts_with(dir_path) {
                        return true;
                    }
                    if let Ok(rel) = dir_path.strip_prefix(root) {
                        if f_path.starts_with(rel) {
                            return true;
                        }
                    }
                }
            }
            false
        };

        if check_list(&self.staged) {
            return Some(FileChangeStatus::Staged);
        }
        if check_list(&self.unstaged) {
            return Some(FileChangeStatus::Modified);
        }
        if check_list(&self.untracked) {
            return Some(FileChangeStatus::Untracked);
        }

        None
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
                        'M' => FileChangeStatus::Staged,
                        'A' => FileChangeStatus::Added,
                        'D' => FileChangeStatus::Deleted,
                        'R' => FileChangeStatus::Renamed,
                        'C' => FileChangeStatus::Copied,
                        'U' => FileChangeStatus::Conflicted,
                        _ => FileChangeStatus::Staged,
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

    /// Fetch git commit log across all branches using OmniGit's robust ASCII unit separator format
    pub fn get_log(cwd: &Path, max_count: usize) -> Result<Vec<GitCommit>> {
        let count_str = max_count.to_string();
        let format_str = format!(
            "%H{f}%h{f}%P{f}%s{f}%b{f}%an{f}%ae{f}%at{f}%D{c}",
            f = FIELD_SEP,
            c = COMMIT_SEP
        );

        let res = Self::exec(
            cwd,
            &[
                "log",
                "--all",
                "--date-order",
                &format!("--format={}", format_str),
                "-n",
                &count_str,
            ],
        );

        match res {
            Ok(raw) => Self::parse_log(&raw),
            Err(err) => {
                let err_str = err.to_string();
                if err_str.contains("does not have any commits")
                    || err_str.contains("unknown revision")
                    || err_str.contains("bad default revision")
                {
                    // Fresh or empty repository
                    Ok(Vec::new())
                } else {
                    // Fallback to simple log without --all in case of shallow or special clones
                    if let Ok(raw) = Self::exec(
                        cwd,
                        &["log", &format!("--format={}", format_str), "-n", &count_str],
                    ) {
                        Self::parse_log(&raw)
                    } else {
                        Err(err)
                    }
                }
            }
        }
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
            if fields.len() < 8 {
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
            let refs = if fields.len() > 8 {
                fields[8]
                    .split(',')
                    .map(|r| r.trim().to_string())
                    .filter(|r| !r.is_empty())
                    .collect()
            } else {
                Vec::new()
            };

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

    /// Get list of changed files for a specific commit (status and path)
    pub fn get_commit_files(cwd: &Path, commit_hash: &str) -> Result<Vec<(String, String)>> {
        let raw = Self::exec(cwd, &["show", "--name-status", "--oneline", commit_hash])?;

        let mut files = Vec::new();
        // Skip first line (oneline commit summary)
        for line in raw.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some((st, path)) = line.split_once('\t') {
                files.push((st.trim().to_string(), path.trim().trim_matches('"').to_string()));
            } else if let Some((st, path)) = line.split_once(' ') {
                files.push((st.trim().to_string(), path.trim().trim_matches('"').to_string()));
            }
        }
        Ok(files)
    }

    /// Compute topological graph lanes for visual commit graph rendering
    pub fn compute_graph_nodes(commits: &[GitCommit]) -> Vec<GitGraphNode> {
        let mut nodes = Vec::with_capacity(commits.len());
        let mut active_lanes: Vec<Option<String>> = Vec::new();

        for commit in commits {
            // 1. Check if this commit's hash is already reserved in active_lanes
            let mut lane = None;
            for (idx, slot) in active_lanes.iter().enumerate() {
                if slot.as_deref() == Some(&commit.hash) {
                    lane = Some(idx);
                    break;
                }
            }

            // If not found in active lanes, find first None slot or append
            let lane = lane.unwrap_or_else(|| {
                for (idx, slot) in active_lanes.iter().enumerate() {
                    if slot.is_none() {
                        return idx;
                    }
                }
                active_lanes.push(None);
                active_lanes.len() - 1
            });

            if lane >= active_lanes.len() {
                active_lanes.resize(lane + 1, None);
            }

            let total_lanes = active_lanes.len().max(1);
            let mut active_mask = vec![false; total_lanes];
            for (i, slot) in active_lanes.iter().enumerate() {
                if slot.is_some() || i == lane {
                    active_mask[i] = true;
                }
            }

            let is_head = commit
                .refs
                .iter()
                .any(|r| r.starts_with("HEAD") || r.contains("HEAD ->"));
            let is_merge = commit.parents.len() > 1;
            let is_root = commit.parents.is_empty();

            // 2. Update active_lanes with parents
            if is_root {
                active_lanes[lane] = None;
            } else {
                active_lanes[lane] = Some(commit.parents[0].clone());
                for extra_parent in commit.parents.iter().skip(1) {
                    let mut found_slot = false;
                    for slot in active_lanes.iter_mut() {
                        if slot.is_none() {
                            *slot = Some(extra_parent.clone());
                            found_slot = true;
                            break;
                        }
                    }
                    if !found_slot {
                        active_lanes.push(Some(extra_parent.clone()));
                    }
                }
            }

            // Trim trailing None slots
            while active_lanes.last().map(|s| s.is_none()).unwrap_or(false) {
                active_lanes.pop();
            }

            nodes.push(GitGraphNode {
                commit: commit.clone(),
                lane,
                total_lanes: total_lanes.max(1),
                active_lanes: active_mask,
                is_head,
                is_merge,
                is_root,
            });
        }

        nodes
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
        assert_eq!(status.staged[0].status, FileChangeStatus::Staged);
        assert_eq!(status.staged[0].status.badge(), "S");
        assert!(status.staged[0].is_staged);

        assert_eq!(status.unstaged.len(), 1);
        assert_eq!(status.unstaged[0].path, "crates/codium-ui/src/views/activity_bar.rs");
        assert_eq!(status.unstaged[0].status, FileChangeStatus::Modified);
        assert_eq!(status.unstaged[0].status.badge(), "M");
        assert!(!status.unstaged[0].is_staged);

        assert_eq!(status.untracked.len(), 1);
        assert_eq!(status.untracked[0].path, "settings/keymaps/vscode.json");
        assert_eq!(status.untracked[0].status, FileChangeStatus::Untracked);
        assert_eq!(status.untracked[0].status.badge(), "U");

        // Test get_file_change helper
        let staged_change = status.get_file_change("crates/codium-core/src/lib.rs").unwrap();
        assert_eq!(staged_change.status, FileChangeStatus::Staged);
        let unstaged_change = status.get_file_change("crates/codium-ui/src/views/activity_bar.rs").unwrap();
        assert_eq!(unstaged_change.status, FileChangeStatus::Modified);
        let untracked_change = status.get_file_change("settings/keymaps/vscode.json").unwrap();
        assert_eq!(untracked_change.status, FileChangeStatus::Untracked);
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

            let commits = GitExecutor::get_log(cwd, 20).unwrap();
            println!("Fetched commits count: {}", commits.len());
            for c in &commits {
                println!("Commit: {} - {} (refs: {:?})", c.short_hash, c.subject, c.refs);
            }
            assert!(!commits.is_empty());

            let nodes = GitExecutor::compute_graph_nodes(&commits);
            assert_eq!(nodes.len(), commits.len());
            assert_eq!(nodes[0].lane, 0);
            assert!(nodes[0].is_head);
        }
    }

    #[test]
    fn test_compute_graph_nodes_mock() {
        let commits = vec![
            GitCommit {
                hash: "commit3".to_string(),
                short_hash: "c3".to_string(),
                parents: vec!["commit2".to_string()],
                subject: "Third".to_string(),
                body: "".to_string(),
                author_name: "Dev".to_string(),
                author_email: "dev@example.com".to_string(),
                author_date: 1000,
                refs: vec!["HEAD -> master".to_string()],
            },
            GitCommit {
                hash: "commit2".to_string(),
                short_hash: "c2".to_string(),
                parents: vec!["commit1".to_string()],
                subject: "Second".to_string(),
                body: "".to_string(),
                author_name: "Dev".to_string(),
                author_email: "dev@example.com".to_string(),
                author_date: 900,
                refs: vec![],
            },
            GitCommit {
                hash: "commit1".to_string(),
                short_hash: "c1".to_string(),
                parents: vec![],
                subject: "First (root)".to_string(),
                body: "".to_string(),
                author_name: "Dev".to_string(),
                author_email: "dev@example.com".to_string(),
                author_date: 800,
                refs: vec![],
            },
        ];

        let nodes = GitExecutor::compute_graph_nodes(&commits);
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].lane, 0);
        assert!(nodes[0].is_head);
        assert!(!nodes[0].is_root);

        assert_eq!(nodes[1].lane, 0);
        assert!(!nodes[1].is_head);
        assert!(!nodes[1].is_root);

        assert_eq!(nodes[2].lane, 0);
        assert!(!nodes[2].is_head);
        assert!(nodes[2].is_root);
    }

    #[test]
    fn test_find_file_change_and_dir_has_changes() {
        let repo_root = PathBuf::from("/workspace/myproject");
        let status = GitRepoStatus {
            is_repo: true,
            root_dir: Some(repo_root.clone()),
            current_branch: "master".to_string(),
            ahead: 0,
            behind: 0,
            staged: vec![
                GitFileChange {
                    path: "crates/core/src/lib.rs".to_string(),
                    status: FileChangeStatus::Staged,
                    is_staged: true,
                    old_path: None,
                },
            ],
            unstaged: vec![
                GitFileChange {
                    path: "crates/ui/src/main.rs".to_string(),
                    status: FileChangeStatus::Modified,
                    is_staged: false,
                    old_path: None,
                },
            ],
            untracked: vec![
                GitFileChange {
                    path: "docs/readme.txt".to_string(),
                    status: FileChangeStatus::Untracked,
                    is_staged: false,
                    old_path: None,
                },
            ],
        };

        // 1. Test finding staged file with absolute path
        let abs_path = repo_root.join("crates/core/src/lib.rs");
        let change = status.find_file_change(&abs_path, Some(&repo_root));
        assert!(change.is_some());
        let c = change.unwrap();
        assert_eq!(c.status, FileChangeStatus::Staged);
        assert_eq!(c.status.badge(), "S");

        // 2. Test finding unstaged file with relative path
        let rel_path = Path::new("crates/ui/src/main.rs");
        let change = status.find_file_change(rel_path, Some(&repo_root));
        assert!(change.is_some());
        assert_eq!(change.unwrap().status, FileChangeStatus::Modified);

        // 3. Test finding untracked file via suffix
        let sub_path = Path::new("/different/prefix/docs/readme.txt");
        let change = status.find_file_change(sub_path, None);
        assert!(change.is_some());
        assert_eq!(change.unwrap().status, FileChangeStatus::Untracked);

        // 4. Test directory has changes
        assert_eq!(
            status.dir_has_changes(Path::new("crates"), Some(&repo_root)),
            Some(FileChangeStatus::Staged)
        );
        assert_eq!(
            status.dir_has_changes(Path::new("crates/ui"), Some(&repo_root)),
            Some(FileChangeStatus::Modified)
        );
        assert_eq!(
            status.dir_has_changes(Path::new("docs"), Some(&repo_root)),
            Some(FileChangeStatus::Untracked)
        );
        assert_eq!(
            status.dir_has_changes(Path::new("non_existent"), Some(&repo_root)),
            None
        );
    }
}
