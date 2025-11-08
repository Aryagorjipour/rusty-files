use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub fn hash_file<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(65536, file);
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn hash_string(text: &str) -> String {
    hash_bytes(text.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes() {
        let data = b"Hello, world!";
        let hash = hash_bytes(data);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_string() {
        let text = "Hello, world!";
        let hash = hash_string(text);
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_consistency() {
        let data = b"test data";
        let hash1 = hash_bytes(data);
        let hash2 = hash_bytes(data);
        assert_eq!(hash1, hash2);
    }
}
