# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub Actions workflow for automated releases
- Comprehensive CHANGELOG.md for tracking changes

## [0.2.0] - 2025-11-15

### Added
- Initial release of Rusty Files
- Fast file indexing with multi-threaded directory traversal (10,000+ files/second on SSD)
- Multiple search modes: exact, case-insensitive, fuzzy, regex, and glob matching
- Advanced filtering by extension, size, modification date
- Content search with full-text search in text files
- Encoding detection for content search
- File system watching with real-time index updates
- Smart ranking of results by relevance, recency, and path depth
- Incremental updates (only index changed files)
- Support for .gitignore patterns and custom exclusion rules
- SQLite-based persistent index with automatic migrations
- Thread-safe concurrent indexing and searching
- Full-featured CLI application with interactive mode
- Query parser with advanced syntax support
- Configuration management (TOML/JSON)
- Comprehensive benchmark suite
- LRU cache for search performance
- Bloom filter for efficient existence checks
- Export functionality (JSON/text formats)

### Performance
- Indexing: ~981 files/sec (100 files), ~1,230 files/sec (500 files)
- Simple search: 1.47ms average
- Pattern search: 95.9µs average
- Fuzzy search: 1.25ms average
- Filtered search: 433.4µs average
- Complex search: 1.22ms average

### Documentation
- Comprehensive README with examples
- API documentation
- CLI usage guide
- Configuration examples
- Architecture overview

[Unreleased]: https://github.com/Aryagorjipour/rusty-files/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Aryagorjipour/rusty-files/releases/tag/v0.2.0
