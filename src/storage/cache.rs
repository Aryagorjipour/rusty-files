use crate::core::types::FileEntry;
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;

pub struct LruCache {
    capacity: usize,
    cache: RwLock<LruCacheInner>,
}

struct LruCacheInner {
    map: HashMap<PathBuf, FileEntry>,
    order: VecDeque<PathBuf>,
}

impl LruCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: RwLock::new(LruCacheInner {
                map: HashMap::with_capacity(capacity),
                order: VecDeque::with_capacity(capacity),
            }),
        }
    }

    pub fn get(&self, path: &PathBuf) -> Option<FileEntry> {
        let mut cache = self.cache.write();

        if let Some(entry) = cache.map.get(path).cloned() {
            if let Some(pos) = cache.order.iter().position(|p| p == path) {
                cache.order.remove(pos);
            }
            cache.order.push_back(path.clone());
            Some(entry)
        } else {
            None
        }
    }

    pub fn insert(&self, path: PathBuf, entry: FileEntry) {
        let mut cache = self.cache.write();

        if cache.map.contains_key(&path) {
            if let Some(pos) = cache.order.iter().position(|p| p == &path) {
                cache.order.remove(pos);
            }
        } else if cache.map.len() >= self.capacity {
            if let Some(old_path) = cache.order.pop_front() {
                cache.map.remove(&old_path);
            }
        }

        cache.map.insert(path.clone(), entry);
        cache.order.push_back(path);
    }

    pub fn remove(&self, path: &PathBuf) -> Option<FileEntry> {
        let mut cache = self.cache.write();

        if let Some(pos) = cache.order.iter().position(|p| p == path) {
            cache.order.remove(pos);
        }

        cache.map.remove(path)
    }

    pub fn clear(&self) {
        let mut cache = self.cache.write();
        cache.map.clear();
        cache.order.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.read().map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cache.read().map.is_empty()
    }

    pub fn contains(&self, path: &PathBuf) -> bool {
        self.cache.read().map.contains_key(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::FileEntry;

    #[test]
    fn test_lru_cache_basic_operations() {
        let cache = LruCache::new(2);
        let path1 = PathBuf::from("/test/file1.txt");
        let path2 = PathBuf::from("/test/file2.txt");

        let entry1 = FileEntry::new(path1.clone());
        let entry2 = FileEntry::new(path2.clone());

        cache.insert(path1.clone(), entry1.clone());
        assert_eq!(cache.len(), 1);

        cache.insert(path2.clone(), entry2.clone());
        assert_eq!(cache.len(), 2);

        assert!(cache.contains(&path1));
        assert!(cache.contains(&path2));
    }

    #[test]
    fn test_lru_cache_eviction() {
        let cache = LruCache::new(2);
        let path1 = PathBuf::from("/test/file1.txt");
        let path2 = PathBuf::from("/test/file2.txt");
        let path3 = PathBuf::from("/test/file3.txt");

        cache.insert(path1.clone(), FileEntry::new(path1.clone()));
        cache.insert(path2.clone(), FileEntry::new(path2.clone()));
        cache.insert(path3.clone(), FileEntry::new(path3.clone()));

        assert_eq!(cache.len(), 2);
        assert!(!cache.contains(&path1));
        assert!(cache.contains(&path2));
        assert!(cache.contains(&path3));
    }

    #[test]
    fn test_lru_cache_get_updates_order() {
        let cache = LruCache::new(2);
        let path1 = PathBuf::from("/test/file1.txt");
        let path2 = PathBuf::from("/test/file2.txt");
        let path3 = PathBuf::from("/test/file3.txt");

        cache.insert(path1.clone(), FileEntry::new(path1.clone()));
        cache.insert(path2.clone(), FileEntry::new(path2.clone()));

        cache.get(&path1);

        cache.insert(path3.clone(), FileEntry::new(path3.clone()));

        assert!(cache.contains(&path1));
        assert!(!cache.contains(&path2));
        assert!(cache.contains(&path3));
    }
}
