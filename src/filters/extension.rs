use crate::core::types::FileEntry;

pub fn apply_extension_filter(entry: &FileEntry, extensions: &[String]) -> bool {
    if extensions.is_empty() {
        return true;
    }

    if let Some(ref ext) = entry.extension {
        extensions.iter().any(|e| e.eq_ignore_ascii_case(ext))
    } else {
        false
    }
}

pub fn normalize_extension(ext: &str) -> String {
    ext.trim_start_matches('.').to_lowercase()
}

pub fn parse_extensions(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|s| normalize_extension(s.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn is_source_code_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" | "cc" | "cxx" | "h"
            | "hpp" | "cs" | "go" | "rb" | "php" | "swift" | "kt" | "scala" | "clj" | "hs"
            | "ml" | "ex" | "exs" | "erl" | "vim" | "lua" | "r" | "sh" | "bash" | "zsh"
            | "fish" | "ps1" | "psm1"
    )
}

pub fn is_document_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "txt" | "md" | "pdf" | "doc" | "docx" | "odt" | "rtf" | "tex" | "xls" | "xlsx" | "ods"
            | "ppt" | "pptx" | "odp"
    )
}

pub fn is_image_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff" | "tif"
            | "heic" | "heif"
    )
}

pub fn is_video_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" | "mpg" | "mpeg"
    )
}

pub fn is_audio_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" | "opus" | "ape"
    )
}

pub fn is_archive_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "tgz" | "tbz2" | "txz"
    )
}

pub fn get_extension_category(ext: &str) -> ExtensionCategory {
    if is_source_code_extension(ext) {
        ExtensionCategory::SourceCode
    } else if is_document_extension(ext) {
        ExtensionCategory::Document
    } else if is_image_extension(ext) {
        ExtensionCategory::Image
    } else if is_video_extension(ext) {
        ExtensionCategory::Video
    } else if is_audio_extension(ext) {
        ExtensionCategory::Audio
    } else if is_archive_extension(ext) {
        ExtensionCategory::Archive
    } else {
        ExtensionCategory::Other
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionCategory {
    SourceCode,
    Document,
    Image,
    Video,
    Audio,
    Archive,
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_extension() {
        assert_eq!(normalize_extension(".txt"), "txt");
        assert_eq!(normalize_extension("TXT"), "txt");
        assert_eq!(normalize_extension("rs"), "rs");
    }

    #[test]
    fn test_parse_extensions() {
        let exts = parse_extensions("rs, .txt, .MD");
        assert_eq!(exts, vec!["rs", "txt", "md"]);
    }

    #[test]
    fn test_is_source_code_extension() {
        assert!(is_source_code_extension("rs"));
        assert!(is_source_code_extension("py"));
        assert!(!is_source_code_extension("txt"));
    }

    #[test]
    fn test_get_extension_category() {
        assert_eq!(get_extension_category("rs"), ExtensionCategory::SourceCode);
        assert_eq!(get_extension_category("pdf"), ExtensionCategory::Document);
        assert_eq!(get_extension_category("png"), ExtensionCategory::Image);
        assert_eq!(get_extension_category("mp4"), ExtensionCategory::Video);
        assert_eq!(get_extension_category("mp3"), ExtensionCategory::Audio);
        assert_eq!(get_extension_category("zip"), ExtensionCategory::Archive);
    }
}
