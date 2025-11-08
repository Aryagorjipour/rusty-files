use encoding_rs::{Encoding, UTF_8};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn detect_encoding(data: &[u8]) -> &'static Encoding {
    let (encoding, _) = Encoding::for_bom(data).unwrap_or((UTF_8, 0));

    if encoding == UTF_8 {
        return encoding;
    }

    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(data, true);
    detector.guess(None, true)
}

pub fn read_file_with_encoding<P: AsRef<Path>>(path: P, max_size: u64) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let file_size = file.metadata()?.len();

    let read_size = std::cmp::min(file_size, max_size);
    let mut buffer = vec![0u8; read_size as usize];

    file.read_exact(&mut buffer)?;

    let encoding = detect_encoding(&buffer);
    let (decoded, _, had_errors) = encoding.decode(&buffer);

    if had_errors {
        Ok(String::from_utf8_lossy(&buffer).to_string())
    } else {
        Ok(decoded.to_string())
    }
}

pub fn is_likely_text(data: &[u8]) -> bool {
    if data.is_empty() {
        return true;
    }

    let sample_size = std::cmp::min(data.len(), 8192);
    let sample = &data[..sample_size];

    let null_count = sample.iter().filter(|&&b| b == 0).count();
    if null_count > sample_size / 10 {
        return false;
    }

    let control_count = sample
        .iter()
        .filter(|&&b| b < 32 && b != b'\n' && b != b'\r' && b != b'\t')
        .count();

    // Allow at least 1 control character for small files, or 5% for larger files
    let threshold = std::cmp::max(1, sample_size / 20);
    control_count < threshold
}

pub fn is_utf8(data: &[u8]) -> bool {
    std::str::from_utf8(data).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_likely_text() {
        assert!(is_likely_text(b"Hello, world!"));
        assert!(is_likely_text(b""));
        assert!(!is_likely_text(&[0u8; 100]));
    }

    #[test]
    fn test_is_utf8() {
        assert!(is_utf8(b"Hello, world!"));
        assert!(is_utf8("こんにちは".as_bytes()));
        assert!(!is_utf8(&[0xFF, 0xFE, 0xFD]));
    }
}
