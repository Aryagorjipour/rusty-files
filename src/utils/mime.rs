use mime_guess::MimeGuess;
use std::path::Path;

pub fn detect_mime_type<P: AsRef<Path>>(path: P) -> Option<String> {
    let guess = MimeGuess::from_path(path.as_ref());
    guess.first().map(|m| m.to_string())
}

pub fn is_text_mime(mime: &str) -> bool {
    mime.starts_with("text/") || is_code_mime(mime)
}

pub fn is_code_mime(mime: &str) -> bool {
    matches!(
        mime,
        "application/javascript"
            | "application/json"
            | "application/xml"
            | "application/x-sh"
            | "application/x-python"
            | "application/x-ruby"
            | "application/x-perl"
            | "application/x-php"
    )
}

pub fn is_image_mime(mime: &str) -> bool {
    mime.starts_with("image/")
}

pub fn is_video_mime(mime: &str) -> bool {
    mime.starts_with("video/")
}

pub fn is_audio_mime(mime: &str) -> bool {
    mime.starts_with("audio/")
}

pub fn is_archive_mime(mime: &str) -> bool {
    matches!(
        mime,
        "application/zip"
            | "application/x-tar"
            | "application/gzip"
            | "application/x-bzip2"
            | "application/x-7z-compressed"
            | "application/x-rar-compressed"
    )
}

pub fn categorize_file<P: AsRef<Path>>(path: P) -> FileCategory {
    if let Some(mime) = detect_mime_type(path) {
        if is_text_mime(&mime) || is_code_mime(&mime) {
            FileCategory::Text
        } else if is_image_mime(&mime) {
            FileCategory::Image
        } else if is_video_mime(&mime) {
            FileCategory::Video
        } else if is_audio_mime(&mime) {
            FileCategory::Audio
        } else if is_archive_mime(&mime) {
            FileCategory::Archive
        } else {
            FileCategory::Other
        }
    } else {
        FileCategory::Unknown
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    Text,
    Image,
    Video,
    Audio,
    Archive,
    Other,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_mime_type() {
        assert!(detect_mime_type("test.txt").is_some());
        assert!(detect_mime_type("test.rs").is_some());
        assert!(detect_mime_type("test.png").is_some());
    }

    #[test]
    fn test_is_text_mime() {
        assert!(is_text_mime("text/plain"));
        assert!(is_text_mime("text/html"));
        assert!(!is_text_mime("image/png"));
    }

    #[test]
    fn test_categorize_file() {
        assert_eq!(categorize_file("test.txt"), FileCategory::Text);
        assert_eq!(categorize_file("test.png"), FileCategory::Image);
        assert_eq!(categorize_file("test.mp4"), FileCategory::Video);
        assert_eq!(categorize_file("test.mp3"), FileCategory::Audio);
        assert_eq!(categorize_file("test.zip"), FileCategory::Archive);
    }
}
