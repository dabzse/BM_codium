use std::fs;
use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::buffer::TextBuffer;

#[derive(Clone, Debug)]
pub struct FileNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub children: Vec<FileNode>,
}

impl FileNode {
    pub fn from_path(path: &Path) -> Result<Self> {
        let name = path
            .file_name()
            .map(|f| f.to_string_lossy().into_owned())
            .unwrap_or_else(|| "/".to_string());
        let is_dir = path.is_dir();

        let mut node = Self {
            path: path.to_path_buf(),
            name,
            is_dir,
            is_expanded: false,
            children: Vec::new(),
        };

        if is_dir {
            node.load_children()?;
        }

        Ok(node)
    }

    pub fn load_children(&mut self) -> Result<()> {
        if !self.is_dir {
            return Ok(());
        }

        let mut children = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.path) {
            for entry in entries.flatten() {
                let child_path = entry.path();
                let child_name = child_path
                    .file_name()
                    .map(|f| f.to_string_lossy().into_owned())
                    .unwrap_or_default();

                // Skip hidden files and heavy directories
                if child_name.starts_with('.') || child_name == "target" || child_name == "node_modules" {
                    continue;
                }

                let is_dir = child_path.is_dir();
                children.push(FileNode {
                    path: child_path,
                    name: child_name,
                    is_dir,
                    is_expanded: false,
                    children: Vec::new(),
                });
            }
        }

        // Sort: directories first, then alphabetical
        children.sort_by(|a, b| {
            match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });

        self.children = children;
        Ok(())
    }

    pub fn toggle_expanded(&mut self) {
        if self.is_dir {
            self.is_expanded = !self.is_expanded;
            if self.is_expanded && self.children.is_empty() {
                let _ = self.load_children();
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Workspace {
    pub root_path: Option<PathBuf>,
    pub file_tree: Vec<FileNode>,
    pub open_documents: Vec<TextBuffer>,
    pub active_document_idx: Option<usize>,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            root_path: None,
            file_tree: Vec::new(),
            open_documents: Vec::new(),
            active_document_idx: None,
        }
    }

    pub fn open_folder(&mut self, path: PathBuf) -> Result<()> {
        if path.is_dir() {
            let mut root_node = FileNode::from_path(&path)?;
            root_node.is_expanded = true;
            self.root_path = Some(path);
            self.file_tree = vec![root_node];
        }
        Ok(())
    }

    pub fn open_file(&mut self, path: PathBuf) -> Result<usize> {
        // If already open, switch to it
        if let Some(idx) = self.open_documents.iter().position(|doc| doc.path() == Some(&path)) {
            self.active_document_idx = Some(idx);
            return Ok(idx);
        }

        let content = if path.exists() {
            fs::read_to_string(&path).unwrap_or_default()
        } else {
            String::new()
        };

        let buffer = TextBuffer::from_str(&content, Some(path));
        self.open_documents.push(buffer);
        let new_idx = self.open_documents.len() - 1;
        self.active_document_idx = Some(new_idx);
        Ok(new_idx)
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.open_documents.len() {
            self.open_documents.remove(idx);
            if self.open_documents.is_empty() {
                self.active_document_idx = None;
            } else if let Some(active) = self.active_document_idx {
                if active >= self.open_documents.len() {
                    self.active_document_idx = Some(self.open_documents.len() - 1);
                }
            }
        }
    }

    pub fn active_document(&self) -> Option<&TextBuffer> {
        self.active_document_idx.and_then(|idx| self.open_documents.get(idx))
    }

    pub fn active_document_mut(&mut self) -> Option<&mut TextBuffer> {
        self.active_document_idx.and_then(|idx| self.open_documents.get_mut(idx))
    }

    pub fn save_active_document(&mut self) -> Result<()> {
        if let Some(doc) = self.active_document_mut() {
            if let Some(path) = doc.path() {
                fs::write(path, doc.text())?;
                doc.mark_clean();
            }
        }
        Ok(())
    }
}
