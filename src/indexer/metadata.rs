use crate::core::error::Result;
use crate::core::types::FileEntry;
use crate::utils::mime::detect_mime_type;
use crate::utils::path::is_hidden;
use chrono::{DateTime, TimeZone, Utc};
use std::fs;
use std::path::Path;

pub struct MetadataExtractor;

impl MetadataExtractor {
    pub fn extract<P: AsRef<Path>>(path: P) -> Result<FileEntry> {
        let path = path.as_ref();
        let metadata = fs::metadata(path)?;

        let mut entry = FileEntry::new(path.to_path_buf());

        entry.size = metadata.len();
        entry.is_directory = metadata.is_dir();
        entry.is_hidden = is_hidden(path);

        #[cfg(unix)]
        {
            entry.is_symlink = metadata.file_type().is_symlink();
        }

        #[cfg(windows)]
        {
            entry.is_symlink = metadata.file_type().is_symlink();
        }

        if let Ok(created) = metadata.created() {
            entry.created_at = Self::system_time_to_datetime(created);
        }

        if let Ok(modified) = metadata.modified() {
            entry.modified_at = Self::system_time_to_datetime(modified);
        }

        if let Ok(accessed) = metadata.accessed() {
            entry.accessed_at = Self::system_time_to_datetime(accessed);
        }

        if !entry.is_directory {
            entry.mime_type = detect_mime_type(path);
        }

        let now = Utc::now();
        entry.indexed_at = now;
        entry.last_verified = now;

        Ok(entry)
    }

    pub fn extract_batch<P: AsRef<Path> + Sync>(paths: &[P]) -> Vec<Result<FileEntry>> {
        use rayon::prelude::*;

        paths
            .par_iter()
            .map(|path| Self::extract(path.as_ref()))
            .collect()
    }

    fn system_time_to_datetime(time: std::time::SystemTime) -> Option<DateTime<Utc>> {
        time.duration_since(std::time::UNIX_EPOCH)
            .ok()
            .and_then(|duration| {
                Utc.timestamp_opt(duration.as_secs() as i64, duration.subsec_nanos())
                    .single()
            })
    }

    pub fn is_modified_since<P: AsRef<Path>>(
        path: P,
        since: DateTime<Utc>,
    ) -> Result<bool> {
        let metadata = fs::metadata(path)?;
        if let Ok(modified) = metadata.modified() {
            if let Some(modified_dt) = Self::system_time_to_datetime(modified) {
                return Ok(modified_dt > since);
            }
        }
        Ok(false)
    }

    pub fn get_file_size<P: AsRef<Path>>(path: P) -> Result<u64> {
        let metadata = fs::metadata(path)?;
        Ok(metadata.len())
    }

    pub fn is_readable<P: AsRef<Path>>(path: P) -> bool {
        fs::metadata(path).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_extract_file_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello, world!").unwrap();

        let entry = MetadataExtractor::extract(&file_path).unwrap();

        assert_eq!(entry.name, "test.txt");
        assert_eq!(entry.extension, Some("txt".to_string()));
        assert_eq!(entry.size, 13);
        assert!(!entry.is_directory);
    }

    #[test]
    fn test_extract_directory_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("subdir");
        fs::create_dir(&dir_path).unwrap();

        let entry = MetadataExtractor::extract(&dir_path).unwrap();

        assert_eq!(entry.name, "subdir");
        assert!(entry.is_directory);
    }

    #[test]
    fn test_extract_batch() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();

        let paths = vec![file1, file2];
        let results = MetadataExtractor::extract_batch(&paths);

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
