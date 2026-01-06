# Contributing to PersonaForge

First off, thanks for taking the time to contribute! 🎉

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Pull Request Process](#pull-request-process)
- [Style Guidelines](#style-guidelines)
- [Community](#community)

## 📜 Code of Conduct

This project adheres to a Code of Conduct. By participating, you are expected to uphold this code. Please report unacceptable behavior to the maintainers.

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ ([rustup.rs](https://rustup.rs/))
- Ollama ([ollama.ai](https://ollama.ai/))
- SQLite
- Git

### Types of Contributions

- 🐛 **Bug fixes**: Found a bug? Fix it!
- ✨ **Features**: Have an idea? Implement it!
- 📚 **Documentation**: Improve docs, add examples
- 🧪 **Tests**: Add test coverage
- 🌐 **Translations**: Help translate prompts/messages
- 🎨 **UI/UX**: Improve Mini App interface

## 💻 Development Setup

```bash
# Clone the repository
git clone https://github.com/bobberdolle1/PersonaForge.git
cd PersonaForge

# Copy environment config
cp .env.example .env

# Edit .env with your settings
# At minimum: TELOXIDE_TOKEN, OWNER_ID

# Install Ollama and pull a model
ollama pull llama3.2
ollama pull nomic-embed-text

# Run in development mode
cargo run

# Run with logging
RUST_LOG=debug cargo run
```

### Project Structure

```
src/
├── main.rs              # Entry point
├── config.rs            # Configuration
├── state.rs             # Shared state
├── bot/handlers/        # Telegram handlers
├── db/                  # Database queries
├── llm/                 # Ollama client
├── security/            # Security features
├── voice/               # Whisper integration
├── web/                 # Web search
└── webapp/              # Mini App
```

## 🔧 Making Changes

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation
- `refactor/description` - Code refactoring

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description

[optional body]

[optional footer]
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation
- `style`: Formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance

Examples:
```
feat(personas): add trigger keywords support
fix(rag): correct time-decay calculation
docs(readme): add Mini App setup guide
```

### Before Submitting

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run tests
cargo test

# Check for security issues
cargo audit
```

## 📤 Pull Request Process

1. **Fork** the repository
2. **Create** a feature branch from `main`
3. **Make** your changes
4. **Test** thoroughly
5. **Update** documentation if needed
6. **Submit** a pull request

### PR Checklist

- [ ] Code follows project style
- [ ] Self-reviewed the code
- [ ] Added comments where needed
- [ ] Updated documentation
- [ ] No new warnings
- [ ] Tests pass locally
- [ ] Commit messages follow convention

### Review Process

1. Maintainers will review your PR
2. Address any requested changes
3. Once approved, PR will be merged
4. Your contribution will be in the next release! 🎉

## 📝 Style Guidelines

### Rust Style

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Address all `clippy` warnings
- Document public APIs with `///` comments

```rust
/// Creates a new persona with the given name and prompt.
///
/// # Arguments
/// * `name` - The persona's display name
/// * `prompt` - The system prompt defining personality
///
/// # Returns
/// The created persona's ID
pub async fn create_persona(name: &str, prompt: &str) -> Result<i64> {
    // Implementation
}
```

### Error Handling

- Use `anyhow::Result` for application errors
- Provide context with `.context("description")`
- Log errors appropriately

```rust
let result = some_operation()
    .await
    .context("Failed to perform operation")?;
```

### Logging

- Use `tracing` macros
- Include relevant context
- Choose appropriate levels:
  - `error!` - Failures requiring attention
  - `warn!` - Potential issues
  - `info!` - Important events
  - `debug!` - Development details
  - `trace!` - Verbose debugging

## 🌟 Recognition

Contributors are recognized in:
- GitHub contributors page
- Release notes
- Special thanks in README (for significant contributions)

## 💬 Community

- **Discussions**: Use GitHub Discussions for questions
- **Issues**: Report bugs and request features
- **Pull Requests**: Submit your contributions

---

Thank you for contributing to PersonaForge! 🤖❤️
