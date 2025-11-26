# Release Process

This document describes the steps to release a new version of dotenvx.

## Prerequisites

Before releasing, ensure you have the following secrets configured in GitHub repository settings:

1. **RELEASE_GITHUB_TOKEN** - GitHub token with `repo` permissions for creating releases
2. **PUBLISH_CARGO_TOKEN** - Token from [crates.io](https://crates.io/me) for publishing to cargo
3. **HOMEBREW_TAP_TOKEN** - GitHub token with write access to [fabianopinto/homebrew-tap](https://github.com/fabianopinto/homebrew-tap)

## Release Steps

### 1. Update Version Number

Edit `Cargo.toml` and bump the version:

```toml
[package]
version = "0.2.0"  # Change from current version
```

### 2. Update Changelog (if applicable)

- Update `README.md` if there are new features
- Document breaking changes
- Update any version references in documentation

### 3. Run Pre-Release Checks

```bash
# Ensure all tests pass
cargo test

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Format code
cargo fmt --all

# Test the build
cargo build --release

# Verify packaging works
cargo package --allow-dirty
```

### 4. Commit Version Changes

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.2.0"
git push origin main
```

### 5. Create and Push Git Tag

```bash
# Create annotated tag (v prefix triggers the release workflow)
git tag -a v0.2.0 -m "Release v0.2.0"

# Push the tag to GitHub
git push origin v0.2.0
```

### 6. Automated Release Workflow

Once you push the tag, the GitHub Actions workflow automatically:

#### **Create GitHub Release** (`create-release` job)

- Creates release with tag name
- Sets release name as "Release v0.2.0"

#### **Build Multi-Platform Binaries** (`build` job)

Builds for the following targets:

- Linux (x86_64-unknown-linux-gnu)
- Linux MUSL (x86_64-unknown-linux-musl)
- macOS Intel (x86_64-apple-darwin)
- macOS ARM (aarch64-apple-darwin)
- Windows (x86_64-pc-windows-msvc)

Creates `.tar.gz` archives (Unix) or `.zip` (Windows) and uploads to GitHub Release.

#### **Publish to crates.io** (`publish-crate` job)

- Runs `cargo publish` with your `PUBLISH_CARGO_TOKEN`
- Makes package available on crates.io

#### **Publish to Homebrew** (`publish-homebrew` job)

- Downloads macOS binaries (Intel and ARM)
- Calculates SHA256 checksums
- Updates the Homebrew formula in [fabianopinto/homebrew-tap](https://github.com/fabianopinto/homebrew-tap)
- Commits and pushes the updated formula
- Makes package available via `brew install fabianopinto/tap/dotenvx`

### 7. Verify Release

After the workflow completes (~10-15 minutes):

#### **GitHub Release**

Check [github.com/fabianopinto/dotenvx/releases](https://github.com/fabianopinto/dotenvx/releases)

- Verify all binary artifacts are uploaded
- Add release notes if needed (edit the release)

#### **crates.io**

Check [crates.io/crates/dotenvx](https://crates.io/crates/dotenvx)

- New version should appear within minutes
- Verify documentation is generated at docs.rs

#### **Homebrew**

Check [github.com/fabianopinto/homebrew-tap](https://github.com/fabianopinto/homebrew-tap)

- Verify the formula was updated
- Test installation: `brew install fabianopinto/tap/dotenvx`

### 8. Post-Release Testing

Test installation from each distribution channel:

```bash
# Test crates.io
cargo install dotenvx

# Test Homebrew (macOS)
brew tap fabianopinto/tap
brew install dotenvx

# Test binary download
wget https://github.com/fabianopinto/dotenvx/releases/download/v0.2.0/dotenvx-x86_64-unknown-linux-gnu.tar.gz
tar xzf dotenvx-x86_64-unknown-linux-gnu.tar.gz
./dotenvx --version
```

## Troubleshooting

### GitHub Release Fails

- Check GitHub Actions logs for errors
- Ensure `RELEASE_GITHUB_TOKEN` has proper permissions

### crates.io Publish Fails

- Verify `PUBLISH_CARGO_TOKEN` is set correctly
- Ensure version in `Cargo.toml` is higher than published version
- Check for missing required metadata in `Cargo.toml`
- Verify `cargo package` succeeds locally first

### Homebrew Publish Fails

- Verify `HOMEBREW_TAP_TOKEN` has write access to homebrew-tap repo
- Check that binaries were successfully uploaded to GitHub release
- Ensure the formula template is valid Ruby syntax
- Check GitHub Actions logs for download or SHA256 calculation errors

### Common Issues

#### Version already exists on crates.io

Increment version further and create a new tag.

#### Missing dependencies

Ensure all dependencies are available on crates.io.

#### Package too large

Check that unnecessary files are excluded via `.gitignore` or `Cargo.toml` exclude field.

#### Homebrew formula syntax error

The formula must be valid Ruby. Test locally:

```bash
brew install --build-from-source Formula/dotenvx.rb
```

## Quick Reference

Complete release in one command:

```bash
VERSION="0.2.0"
cargo test && \
  sed -i '' "s/^version = .*/version = \"$VERSION\"/" Cargo.toml && \
  git add Cargo.toml Cargo.lock && \
  git commit -m "chore: bump version to $VERSION" && \
  git push origin main && \
  git tag -a "v$VERSION" -m "Release v$VERSION" && \
  git push origin "v$VERSION"
```

The automated workflow handles:

- ✅ GitHub Release creation
- ✅ Multi-platform binary builds
- ✅ crates.io publishing
- ✅ Homebrew tap update

All done automatically! 🎉
