# Contributing to dotenvx

Thank you for considering contributing to dotenvx! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.70 or later (install via [rustup](https://rustup.rs/))
- Git
- A GitHub account

### Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:

```bash
git clone https://github.com/YOUR_USERNAME/dotenvx.git
cd dotenvx
```

3. Add the upstream repository:

```bash
git remote add upstream https://github.com/fabianopinto/dotenvx.git
```

4. Create a new branch for your changes:

```bash
git checkout -b feature/my-new-feature
```

## Building and Testing

### Build the project

```bash
cargo build
```

### Run tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Run clippy (linter)

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

### Format code

```bash
cargo fmt --all
```

### Check formatting

```bash
cargo fmt --all -- --check
```

## Code Style

- Follow Rust standard formatting (use `cargo fmt`)
- Write idiomatic Rust code
- Add documentation comments for public APIs
- Keep functions small and focused
- Use meaningful variable names

### Documentation

- Add rustdoc comments for all public items
- Include examples in documentation where appropriate
- Update README.md if adding new features

Example:

```rust
/// Encrypt a value using ECIES
///
/// # Arguments
///
/// * `plaintext` - The value to encrypt
/// * `public_key_hex` - The hex-encoded public key
///
/// # Returns
///
/// The encrypted value with "encrypted:" prefix
///
/// # Example
///
/// ```
/// use dotenvx::crypto::encrypt;
///
/// let encrypted = encrypt("secret", "034af93e...").unwrap();
/// assert!(encrypted.starts_with("encrypted:"));
/// ```
pub fn encrypt(plaintext: &str, public_key_hex: &str) -> Result<String> {
    // Implementation
}
```

## Testing Guidelines

### Test Coverage

- Aim for >85% code coverage
- Write unit tests for all modules
- Write integration tests for CLI commands
- Include edge cases and error conditions

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = "test";

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }

    #[test]
    fn test_error_case() {
        let result = function_that_should_fail("invalid");
        assert!(result.is_err());
    }
}
```

## Submitting Changes

### Before Submitting

1. Ensure all tests pass: `cargo test`
2. Run clippy: `cargo clippy -- -D warnings`
3. Format code: `cargo fmt`
4. Update documentation if needed
5. Add/update tests for your changes

### Commit Messages

Follow conventional commits format:

```
type(scope): brief description

Longer description if needed.

Fixes #123
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Test additions/changes
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `chore`: Build process or tooling changes

Example:

```
feat(crypto): add AES-256-GCM encryption support

Add AES-256-GCM encryption as an alternative to ECIES
for certain use cases where performance is critical.

Fixes #42
```

### Pull Request Process

1. Update your branch with upstream changes:

```bash
git fetch upstream
git rebase upstream/main
```

2. Push to your fork:

```bash
git push origin feature/my-new-feature
```

3. Open a pull request on GitHub

4. Ensure CI passes (tests, clippy, formatting)

5. Address review feedback

6. Once approved, a maintainer will merge your PR

### Pull Request Guidelines

- Keep PRs focused on a single feature or fix
- Include tests for new functionality
- Update documentation as needed
- Reference related issues
- Respond to feedback promptly

## Reporting Issues

### Bug Reports

Include:
- dotenvx version (`dotenvx --version`)
- Operating system and version
- Rust version (`rustc --version`)
- Steps to reproduce
- Expected behavior
- Actual behavior
- Error messages (if any)

### Feature Requests

Include:
- Clear description of the feature
- Use cases and examples
- Potential implementation approach (optional)

## Code of Conduct

### Our Standards

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive criticism
- Accept responsibility for mistakes
- Prioritize community benefit

### Unacceptable Behavior

- Harassment or discrimination
- Trolling or insulting comments
- Public or private harassment
- Publishing others' private information

## Questions?

- Open an issue for questions
- Join discussions on GitHub
- Check existing issues and PRs

## License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).
