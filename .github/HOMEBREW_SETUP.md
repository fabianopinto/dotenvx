# Homebrew Tap Setup

This document describes the Homebrew tap integration for dotenvx.

## Overview

The release workflow automatically publishes dotenvx to the Homebrew tap at:
**https://github.com/fabianopinto/homebrew-tap**

## Required Secret

Add the following secret to your GitHub repository settings:

- **Name**: `HOMEBREW_TAP_TOKEN`
- **Value**: A GitHub Personal Access Token with write access to `fabianopinto/homebrew-tap`
- **Location**: Repository Settings → Secrets and variables → Actions → Repository secrets

### Creating the Token

1. Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
2. Click "Generate new token (classic)"
3. Give it a descriptive name: "dotenvx Homebrew Publishing"
4. Select scopes:
   - ✅ `repo` (Full control of private repositories)
5. Set expiration as needed
6. Generate and copy the token
7. Add it to the dotenvx repository secrets as `HOMEBREW_TAP_TOKEN`

## What the Workflow Does

The `publish-homebrew` job in `.github/workflows/release.yml`:

1. **Downloads macOS binaries** - Both Intel (x86_64) and ARM (aarch64) builds
2. **Calculates SHA256 checksums** - For both binaries
3. **Clones the homebrew-tap repository** - Using the `HOMEBREW_TAP_TOKEN`
4. **Generates the Homebrew formula** - Creates `Formula/dotenvx.rb` with:
   - Version information
   - Download URLs for both architectures
   - SHA256 checksums for verification
   - Installation instructions
   - Test commands
5. **Commits and pushes** - Automatically updates the tap repository

## Formula Structure

The generated formula looks like this:

```ruby
class Dotenvx < Formula
  desc "A secure environment variable management tool with built-in encryption"
  homepage "https://github.com/fabianopinto/dotenvx"
  version "0.2.0"
  license "MIT OR Apache-2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/fabianopinto/dotenvx/releases/download/v0.2.0/dotenvx-aarch64-apple-darwin.tar.gz"
      sha256 "abc123..."
    else
      url "https://github.com/fabianopinto/dotenvx/releases/download/v0.2.0/dotenvx-x86_64-apple-darwin.tar.gz"
      sha256 "def456..."
    end
  end

  def install
    bin.install "dotenvx"
  end

  test do
    system "#{bin}/dotenvx", "--version"
  end
end
```

## User Installation

Once published, users can install dotenvx via Homebrew:

```bash
# Add the tap
brew tap fabianopinto/tap

# Install dotenvx
brew install dotenvx

# Or in one command
brew install fabianopinto/tap/dotenvx
```

## Updating

Updates are automatic. When you create a new release:

1. Push a new version tag (e.g., `v0.2.0`)
2. GitHub Actions runs the release workflow
3. The Homebrew formula is automatically updated
4. Users can upgrade with: `brew upgrade dotenvx`

## Troubleshooting

### Token permissions error

- Ensure `HOMEBREW_TAP_TOKEN` has write access to the tap repository
- Regenerate the token with correct `repo` scope

### Binary download fails

- Check that the GitHub release was created successfully
- Verify the binary filenames match the expected pattern

### SHA256 mismatch

- This usually means the binaries were corrupted or changed
- Re-run the release workflow

### Formula syntax error

- Test the formula locally: `brew install --build-from-source Formula/dotenvx.rb`
- Check the Ruby syntax in the generated formula
- Verify the sed replacements in the workflow

## Manual Formula Update

If needed, you can manually update the formula:

```bash
# Clone the tap
git clone git@github.com:fabianopinto/homebrew-tap.git
cd homebrew-tap

# Edit the formula
vim Formula/dotenvx.rb

# Test it
brew install --build-from-source Formula/dotenvx.rb

# Commit and push
git add Formula/dotenvx.rb
git commit -m "Update dotenvx formula"
git push origin main
```

## Testing

Test the formula locally before release:

```bash
# Audit the formula
brew audit --strict Formula/dotenvx.rb

# Install from the formula
brew install --build-from-source Formula/dotenvx.rb

# Test the installation
dotenvx --version
```
