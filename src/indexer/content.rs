use crate::core::error::Result;
use crate::core::types::ContentPreview;
use crate::utils::encoding::{detect_encoding, is_likely_text, read_file_with_encoding};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct ContentAnalyzer {
    max_file_size: u64,
    preview_length: usize,
}

impl ContentAnalyzer {
    pub fn new(max_file_size: u64) -> Self {
        Self {
            max_file_size,
            preview_length: 1000,
        }
    }

    pub fn analyze<P: AsRef<Path>>(&self, path: P) -> Result<Option<ContentPreview>> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;

        if metadata.len() > self.max_file_size {
            return Ok(None);
        }

        if !self.is_text_file(path)? {
            return Ok(None);
        }

        let content = read_file_with_encoding(path, self.max_file_size)?;

        let preview = if content.len() > self.preview_length {
            content.chars().take(self.preview_length).collect()
        } else {
            content.clone()
        };

        let word_count = content.split_whitespace().count();
        let line_count = content.lines().count();

        let mut file = File::open(path)?;
        let mut buffer = vec![0u8; 8192.min(metadata.len() as usize)];
        file.read_exact(&mut buffer)?;

        let encoding = detect_encoding(&buffer);

        Ok(Some(ContentPreview {
            preview,
            word_count,
            line_count,
            encoding: encoding.name().to_string(),
        }))
    }

    pub fn analyze_batch<P: AsRef<Path> + Sync>(
        &self,
        paths: &[P],
    ) -> Vec<(usize, Result<Option<ContentPreview>>)> {
        use rayon::prelude::*;

        paths
            .par_iter()
            .enumerate()
            .map(|(idx, path)| (idx, self.analyze(path.as_ref())))
            .collect()
    }

    fn is_text_file<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        let mut file = File::open(path)?;
        let mut buffer = vec![0u8; 8192];

        let bytes_read = file.read(&mut buffer)?;
        buffer.truncate(bytes_read);

        Ok(is_likely_text(&buffer))
    }

    pub fn extract_text<P: AsRef<Path>>(&self, path: P, max_length: usize) -> Result<String> {
        let content = read_file_with_encoding(path, self.max_file_size)?;

        if content.len() > max_length {
            Ok(content.chars().take(max_length).collect())
        } else {
            Ok(content)
        }
    }

    pub fn get_snippet<P: AsRef<Path>>(
        &self,
        path: P,
        query: &str,
        context_chars: usize,
    ) -> Result<Option<String>> {
        let content = read_file_with_encoding(path, self.max_file_size)?;

        if let Some(pos) = content.to_lowercase().find(&query.to_lowercase()) {
            let start = pos.saturating_sub(context_chars);
            let end = (pos + query.len() + context_chars).min(content.len());

            let snippet: String = content.chars().skip(start).take(end - start).collect();
            Ok(Some(snippet))
        } else {
            Ok(None)
        }
    }
}

impl Default for ContentAnalyzer {
    fn default() -> Self {
        Self::new(10 * 1024 * 1024)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_analyze_text_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello world\nThis is a test\nWith multiple lines").unwrap();

        let analyzer = ContentAnalyzer::default();
        let preview = analyzer.analyze(&file_path).unwrap();

        assert!(preview.is_some());
        let preview = preview.unwrap();
        // Count: Hello, world, This, is, a, test, With, multiple, lines = 9 words
        assert_eq!(preview.word_count, 9);
        assert_eq!(preview.line_count, 3);
    }

    #[test]
    fn test_analyze_binary_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("binary.bin");
        fs::write(&file_path, vec![0u8; 100]).unwrap();

        let analyzer = ContentAnalyzer::default();
        let preview = analyzer.analyze(&file_path).unwrap();

        assert!(preview.is_none());
    }

    #[test]
    fn test_get_snippet() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "The quick brown fox jumps over the lazy dog").unwrap();

        let analyzer = ContentAnalyzer::default();
        let snippet = analyzer.get_snippet(&file_path, "brown", 10).unwrap();

        assert!(snippet.is_some());
        let snippet = snippet.unwrap();
        assert!(snippet.contains("brown"));
    }
}
