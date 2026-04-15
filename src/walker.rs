use std::path::Path;

use swc_ecma_parser::{EsSyntax, Syntax, TsSyntax};
use walkdir::WalkDir;

use crate::types::FileType;

pub fn collect_files(root_path: &Path) -> Vec<FileType> {
    let mut files: Vec<FileType> = Vec::new();

    for entry in WalkDir::new(root_path)
        .into_iter()
        .filter_entry(|e| !should_skip_dir(e.path()))
        .filter_map(|e| e.ok())
    {
        match entry
            .path()
            .extension()
            .and_then(|s: &std::ffi::OsStr| s.to_str())
        {
            Some("ts") | Some("tsx") => {
                let relative_path = entry
                    .path()
                    .strip_prefix(root_path)
                    .unwrap_or(entry.path())
                    .to_path_buf();

                files.push(FileType {
                    path: entry.path().to_path_buf(),
                    relative_path,
                    is_ts: true,
                });
            }
            Some("js") | Some("jsx") => {
                let relative_path = entry
                    .path()
                    .strip_prefix(root_path)
                    .unwrap_or(entry.path())
                    .to_path_buf();

                files.push(FileType {
                    path: entry.path().to_path_buf(),
                    relative_path,
                    is_ts: false,
                });
            }
            _ => {}
        }
    }

    files
}

pub fn get_syntax(file: &FileType) -> Syntax {
    if file.is_ts {
        Syntax::Typescript(TsSyntax {
            tsx: true,
            decorators: true,
            ..Default::default()
        })
    } else {
        Syntax::Es(EsSyntax {
            jsx: true,
            ..Default::default()
        })
    }
}

fn should_skip_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };

    matches!(name, "node_modules" | ".git" | "target" | "dist")
}
