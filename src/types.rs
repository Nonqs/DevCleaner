use std::path::PathBuf;

pub struct FileType {
    pub path: PathBuf,
    pub relative_path: PathBuf,
    pub is_ts: bool,
}
