# Release Guide

This document provides step-by-step instructions for creating releases for the Rusty Files project.

## Quick Release Workflow

### 1. Prepare the Release

```bash
# Ensure you're on the main branch and up to date
git checkout main
git pull origin main

# Run all tests and benchmarks
cargo test --all-features
cargo bench
cargo fmt --check
cargo clippy --all-features -- -D warnings
```

### 2. Update Version and Changelog

#### Update Cargo.toml
```toml
[package]
version = "0.2.0"  # Update this line
```

#### Update CHANGELOG.md

Add a new section at the top of the file under `[Unreleased]`:

```markdown
## [0.2.0] - 2025-11-08

### Added
- New feature X
- New feature Y

### Changed
- Improved performance of feature Z

### Fixed
- Bug fix A
- Bug fix B

### Performance
- Include relevant benchmark results here
```

Don't forget to update the comparison links at the bottom of CHANGELOG.md:

```markdown
[Unreleased]: https://github.com/Aryagorjipour/rusty-files/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Aryagorjipour/rusty-files/releases/tag/v0.2.0
```

### 3. Commit Version Changes

```bash
git add Cargo.toml CHANGELOG.md
git commit -m "chore: prepare release v0.2.0"
git push origin main
```

### 4. Create and Push Tag

```bash
# Create an annotated tag (recommended)
git tag -a v0.2.0 -m "Release v0.2.0 - Brief description of main features/changes"

# Push the tag (this triggers the release workflow)
git push origin v0.2.0
```

### 5. Monitor the Release Workflow

1. Go to GitHub Actions: https://github.com/Aryagorjipour/rusty-files/actions
2. Watch the "Release" workflow run
3. The workflow will:
   - Run all tests
   - Run benchmarks and extract results
   - Build binaries for Linux, macOS (Intel & ARM), and Windows
   - Generate comprehensive release notes
   - Create a GitHub release with all artifacts

### 6. Verify the Release

1. Check the releases page: https://github.com/Aryagorjipour/rusty-files/releases
2. Verify all binaries are attached
3. Review the release notes
4. Test downloading and running a binary

### 7. Publish to crates.io (Optional)

```bash
# Login to crates.io (first time only)
cargo login

# Publish the crate
cargo publish

# Verify on crates.io
# https://crates.io/crates/rusty-files
```

## Versioning Strategy

Follow [Semantic Versioning 2.0.0](https://semver.org/):

- **MAJOR** version (X.0.0): Incompatible API changes
- **MINOR** version (0.X.0): New functionality, backward-compatible
- **PATCH** version (0.0.X): Bug fixes, backward-compatible

### Examples

- `0.1.0` → `0.1.1`: Bug fixes only
- `0.1.0` → `0.2.0`: New features added
- `0.9.0` → `1.0.0`: First stable release
- `1.5.0` → `2.0.0`: Breaking API changes

### Pre-release Versions

For pre-releases, use:
- `0.2.0-alpha.1`: Alpha releases
- `0.2.0-beta.1`: Beta releases
- `0.2.0-rc.1`: Release candidates

## Release Types

### Patch Release (0.1.X)

**When:** Bug fixes, documentation updates, performance improvements (no new features)

**Steps:**
1. Update CHANGELOG.md with fixes
2. Bump patch version in Cargo.toml
3. Create tag

### Minor Release (0.X.0)

**When:** New features, non-breaking changes

**Steps:**
1. Update CHANGELOG.md with new features
2. Update README.md if needed
3. Bump minor version in Cargo.toml
4. Create tag

### Major Release (X.0.0)

**When:** Breaking changes, major rewrites

**Steps:**
1. Document all breaking changes in CHANGELOG.md
2. Update migration guide
3. Update all documentation
4. Bump major version in Cargo.toml
5. Create tag
6. Consider creating a migration guide document

## Release Checklist

### Pre-Release
- [ ] All tests pass (`cargo test --all-features`)
- [ ] All benchmarks run (`cargo bench`)
- [ ] Code is formatted (`cargo fmt --check`)
- [ ] No clippy warnings (`cargo clippy --all-features -- -D warnings`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated
- [ ] Version is bumped in Cargo.toml
- [ ] README.md reflects new features/changes
- [ ] Examples are tested and working

### Release
- [ ] Version changes are committed
- [ ] Tag is created and pushed
- [ ] GitHub Actions workflow succeeds
- [ ] All binaries are built correctly
- [ ] Release notes are accurate

### Post-Release
- [ ] Release is verified on GitHub
- [ ] Binaries can be downloaded and run
- [ ] Published to crates.io (if applicable)
- [ ] Announcement posted (if applicable)
- [ ] Documentation site updated (if applicable)

## Automated Release Workflow

The `.github/workflows/release.yml` file automates the release process:

### Trigger
Pushing a tag matching `v*.*.*` (e.g., `v0.2.0`)

### Steps
1. **Test & Benchmark**: Runs all tests and benchmarks
2. **Build**: Builds binaries for:
   - Linux (x86_64)
   - macOS (x86_64 and ARM64)
   - Windows (x86_64)
3. **Release**: Creates GitHub release with:
   - Comprehensive release notes
   - Feature list from README
   - Benchmark results
   - Changelog entries
   - Commit history
   - Installation instructions
   - All binary artifacts

### What Gets Included in Release Notes

The automated workflow includes:
- **Highlights**: Overview of the release
- **Features**: Extracted from README.md
- **Changes**: Commit log since last tag
- **Benchmarks**: Actual benchmark results from CI
- **Changelog**: Content from CHANGELOG.md
- **Installation**: Instructions for each platform
- **Quick Start**: Basic usage examples

## Troubleshooting

### Tag Already Exists

```bash
# Delete local tag
git tag -d v0.2.0

# Delete remote tag (be careful!)
git push origin :refs/tags/v0.2.0

# Create new tag
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0
```

### Release Failed

1. Check GitHub Actions logs
2. Fix the issue
3. Delete the tag
4. Re-create and push the tag

### Wrong Version Released

If you released the wrong version:
1. Delete the release on GitHub
2. Delete the tag (local and remote)
3. Fix the version
4. Create a new tag

## Examples

### Example: Patch Release (0.1.0 → 0.1.1)

```bash
# Update Cargo.toml: version = "0.1.1"
# Update CHANGELOG.md with bug fixes

git add Cargo.toml CHANGELOG.md
git commit -m "chore: prepare release v0.1.1"
git push origin main

git tag -a v0.1.1 -m "Release v0.1.1 - Bug fixes"
git push origin v0.1.1
```

### Example: Minor Release (0.1.0 → 0.2.0)

```bash
# Update Cargo.toml: version = "0.2.0"
# Update CHANGELOG.md with new features
# Update README.md with new features

git add Cargo.toml CHANGELOG.md README.md
git commit -m "chore: prepare release v0.2.0"
git push origin main

git tag -a v0.2.0 -m "Release v0.2.0 - Add real-time file watching"
git push origin v0.2.0
```

### Example: Pre-release (0.2.0-beta.1)

```bash
# Update Cargo.toml: version = "0.2.0-beta.1"
# Update CHANGELOG.md

git add Cargo.toml CHANGELOG.md
git commit -m "chore: prepare release v0.2.0-beta.1"
git push origin main

git tag -a v0.2.0-beta.1 -m "Release v0.2.0-beta.1 - Beta release"
git push origin v0.2.0-beta.1
```

## Best Practices

1. **Always use annotated tags**: `git tag -a` not `git tag`
2. **Write descriptive commit messages**: Follow conventional commits
3. **Test before releasing**: Run full test suite and benchmarks
4. **Keep CHANGELOG updated**: Don't let it get stale
5. **Document breaking changes**: Make migration easy
6. **Use semantic versioning**: Be consistent
7. **Review release notes**: Before making release public
8. **Test binaries**: Download and test at least one platform
9. **Announce releases**: Let users know about new features
10. **Keep docs in sync**: Update documentation with releases

## Resources

- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)
- [GitHub Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
- [Cargo Publishing](https://doc.rust-lang.org/cargo/reference/publishing.html)
