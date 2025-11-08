use std::path::{Path, PathBuf};

pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    dunce::canonicalize(path.as_ref()).unwrap_or_else(|_| path.as_ref().to_path_buf())
}

pub fn is_hidden<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name.starts_with('.') && name != "." && name != ".." {
            return true;
        }
    }

    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(metadata) = path.metadata() {
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            return (metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN) != 0;
        }
    }

    false
}

pub fn get_path_depth<P: AsRef<Path>>(path: P) -> usize {
    path.as_ref().components().count()
}

pub fn get_relative_path<P: AsRef<Path>>(base: P, target: P) -> Option<PathBuf> {
    let base = normalize_path(base);
    let target = normalize_path(target);

    pathdiff::diff_paths(&target, &base)
}

pub fn is_same_file<P: AsRef<Path>>(path1: P, path2: P) -> bool {
    let path1 = normalize_path(path1);
    let path2 = normalize_path(path2);

    path1 == path2
}

pub fn get_file_name<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}

pub fn get_file_stem<P: AsRef<Path>>(path: P) -> String {
    path.as_ref()
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string()
}

pub fn get_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_string())
}

pub fn join_paths<P: AsRef<Path>>(base: P, paths: &[P]) -> PathBuf {
    let mut result = base.as_ref().to_path_buf();
    for path in paths {
        result.push(path.as_ref());
    }
    result
}

pub fn ensure_parent_exists<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hidden_unix() {
        assert!(is_hidden(".hidden"));
        assert!(is_hidden("/path/.hidden"));
        assert!(!is_hidden("visible"));
        assert!(!is_hidden("/path/visible"));
    }

    #[test]
    fn test_get_path_depth() {
        assert_eq!(get_path_depth("/"), 1);
        assert_eq!(get_path_depth("/path/to/file"), 4);
    }

    #[test]
    fn test_get_file_name() {
        assert_eq!(get_file_name("/path/to/file.txt"), "file.txt");
        assert_eq!(get_file_name("file.txt"), "file.txt");
    }

    #[test]
    fn test_get_file_stem() {
        assert_eq!(get_file_stem("/path/to/file.txt"), "file");
        assert_eq!(get_file_stem("file.txt"), "file");
    }

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension("/path/to/file.txt"), Some("txt".to_string()));
        assert_eq!(get_extension("file.rs"), Some("rs".to_string()));
        assert_eq!(get_extension("file"), None);
    }
}
