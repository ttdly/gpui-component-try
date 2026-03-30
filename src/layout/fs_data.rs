use std::fs;
use std::path::{Path, PathBuf};

use gpui::SharedString;

use super::right_panel::FileRow;

#[derive(Clone)]
pub struct FsNode {
    pub path: PathBuf,
    pub label: SharedString,
    pub is_dir: bool,
}

pub fn start_dir() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"))
}

pub fn node_for_path(path: &Path) -> Option<FsNode> {
    let metadata = fs::metadata(path).ok()?;

    Some(FsNode {
        path: path.to_path_buf(),
        label: file_name(path),
        is_dir: metadata.is_dir(),
    })
}

pub fn directory_children(path: &Path) -> Vec<FsNode> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let children = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter_map(|child| node_for_path(&child))
        .collect::<Vec<_>>();

    sort_fs_nodes(children)
}

pub fn file_rows_for(path: &Path) -> Vec<FileRow> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut rows = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let metadata = entry.metadata().ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = metadata.is_dir();
            let kind = if is_dir {
                "Folder".to_string()
            } else {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.to_ascii_uppercase())
                    .unwrap_or_else(|| "File".to_string())
            };
            let size = if is_dir {
                "--".to_string()
            } else {
                human_size(metadata.len())
            };

            Some(FileRow::new(path, name, kind, size, is_dir))
        })
        .collect::<Vec<_>>();

    rows.sort_by(|a, b| match b.is_dir().cmp(&a.is_dir()) {
        std::cmp::Ordering::Equal => a.name().to_lowercase().cmp(&b.name().to_lowercase()),
        ordering => ordering,
    });
    rows
}

fn sort_fs_nodes(mut items: Vec<FsNode>) -> Vec<FsNode> {
    items.sort_by(|a, b| match b.is_dir.cmp(&a.is_dir) {
        std::cmp::Ordering::Equal => a.label.to_lowercase().cmp(&b.label.to_lowercase()),
        ordering => ordering,
    });
    items
}

fn file_name(path: &Path) -> SharedString {
    if path.parent().is_none() {
        return "/".into();
    }

    path.file_name()
        .map(|name| name.to_string_lossy().to_string().into())
        .unwrap_or_else(|| path.to_string_lossy().to_string().into())
}

fn human_size(size: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;

    if size as f64 >= MB {
        format!("{:.1} MB", size as f64 / MB)
    } else if size as f64 >= KB {
        format!("{:.1} KB", size as f64 / KB)
    } else {
        format!("{} B", size)
    }
}
