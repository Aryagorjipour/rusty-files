use parking_lot::RwLock;
use probabilistic_collections::bloom::BloomFilter;

pub struct FileBloomFilter {
    filter: RwLock<BloomFilter<String>>,
    capacity: usize,
    error_rate: f64,
}

impl FileBloomFilter {
    pub fn new(capacity: usize, error_rate: f64) -> Self {
        let filter = BloomFilter::new(capacity, error_rate);
        Self {
            filter: RwLock::new(filter),
            capacity,
            error_rate,
        }
    }

    pub fn insert<S: AsRef<str>>(&self, item: S) {
        let mut filter = self.filter.write();
        filter.insert(&item.as_ref().to_string());
    }

    pub fn contains<S: AsRef<str>>(&self, item: S) -> bool {
        let filter = self.filter.read();
        filter.contains(&item.as_ref().to_string())
    }

    pub fn clear(&self) {
        let mut filter = self.filter.write();
        *filter = BloomFilter::new(self.capacity, self.error_rate);
    }

    pub fn len(&self) -> usize {
        self.filter.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.filter.read().is_empty()
    }
}

impl Default for FileBloomFilter {
    fn default() -> Self {
        Self::new(10_000_000, 0.0001)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bloom_filter_basic() {
        let bloom = FileBloomFilter::new(1000, 0.01);

        bloom.insert("test.txt");
        bloom.insert("another.rs");

        assert!(bloom.contains("test.txt"));
        assert!(bloom.contains("another.rs"));
    }

    #[test]
    fn test_bloom_filter_negative_lookup() {
        let bloom = FileBloomFilter::new(1000, 0.01);

        bloom.insert("exists.txt");

        assert!(!bloom.contains("doesnotexist.txt"));
    }

    #[test]
    fn test_bloom_filter_clear() {
        let bloom = FileBloomFilter::new(1000, 0.01);

        bloom.insert("test.txt");
        assert!(bloom.contains("test.txt"));

        bloom.clear();
        assert!(!bloom.contains("test.txt"));
    }
}
