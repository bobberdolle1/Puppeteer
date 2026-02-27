# Contributing to Puppeteer 🎭

Thank you for your interest in contributing to Puppeteer! This document provides guidelines and instructions for contributing.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for everyone.

## How to Contribute

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates. When creating a bug report, include:

- **Clear title and description**
- **Steps to reproduce** the behavior
- **Expected behavior**
- **Actual behavior**
- **Environment details** (OS, Rust version, etc.)
- **Logs or error messages** (if applicable)

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, include:

- **Clear title and description**
- **Use case** explaining why this enhancement would be useful
- **Possible implementation** (if you have ideas)

### Pull Requests

1. **Fork the repository** and create your branch from `main`
2. **Make your changes** following the coding standards below
3. **Add tests** if applicable
4. **Update documentation** if needed
5. **Ensure tests pass**: `cargo test`
6. **Run formatting**: `cargo fmt`
7. **Run linting**: `cargo clippy`
8. **Commit your changes** with clear commit messages
9. **Push to your fork** and submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Docker & Docker Compose (for testing)
- TDLib dependencies (see README)

### Building

```bash
# Clone your fork
git clone https://github.com/yourusername/puppeteer.git
cd puppeteer

# Build
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## Coding Standards

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Address all `cargo clippy` warnings
- Write idiomatic Rust code

### Code Organization

- Keep functions focused and small
- Use meaningful variable and function names
- Add comments for complex logic
- Document public APIs with doc comments

### Error Handling

- Use `Result<T, E>` for fallible operations
- Provide context with `anyhow::Context`
- Log errors appropriately with `tracing`

### Testing

- Write unit tests for new functionality
- Add integration tests for complex features
- Ensure tests are deterministic and isolated

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
feat: add voice message support
fix: resolve database connection leak
docs: update installation instructions
refactor: simplify authentication flow
test: add tests for reply probability
```

## Project Structure

```
src/
├── main.rs              # Application entry point
├── config.rs            # Configuration management
├── state.rs             # Shared application state
├── bot/                 # Admin bot (teloxide)
│   ├── handlers.rs      # Command handlers
│   ├── dialogues.rs     # Authentication flows
│   └── middleware.rs    # Middleware (auth, rate limiting)
├── userbot/             # MTProto userbots
│   └── worker.rs        # Event loop and message handling
├── ai/                  # AI integrations
│   ├── ollama.rs        # LLM client
│   └── whisper.rs       # Voice transcription
└── db/                  # Database layer
    ├── models.rs        # Data models
    └── repository.rs    # Database operations
```

## Areas for Contribution

### High Priority

- [ ] Comprehensive test coverage
- [ ] Performance optimizations
- [ ] Documentation improvements
- [ ] Error handling enhancements

### Features

- [ ] Web dashboard for management
- [ ] Advanced RAG memory system
- [ ] Multi-language support
- [ ] Plugin system
- [ ] Metrics and monitoring

### Infrastructure

- [ ] CI/CD improvements
- [ ] Docker optimization
- [ ] Deployment guides
- [ ] Benchmarking suite

## Questions?

Feel free to:
- Open an issue for discussion
- Join our community discussions
- Reach out to maintainers

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Puppeteer! 🎭
