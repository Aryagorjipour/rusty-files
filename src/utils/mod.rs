pub mod encoding;
pub mod hash;
pub mod mime;
pub mod path;

pub use encoding::{detect_encoding, is_likely_text, is_utf8, read_file_with_encoding};
pub use hash::{hash_bytes, hash_file, hash_string};
pub use mime::{categorize_file, detect_mime_type, FileCategory};
pub use path::{
    ensure_parent_exists, get_extension, get_file_name, get_file_stem, get_path_depth,
    get_relative_path, is_hidden, is_same_file, join_paths, normalize_path,
};
