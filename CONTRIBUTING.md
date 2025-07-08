# Contributing to Cross-Chain Bridge

We welcome contributions to the Cross-Chain Bridge project! This document provides guidelines for contributing.

## 🚀 Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork**: `git clone https://github.com/YOUR_USERNAME/cross-chain-bridge.git`
3. **Create a feature branch**: `git checkout -b feature/your-feature-name`
4. **Make your changes**
5. **Run tests**: `cargo test --workspace`
6. **Commit your changes**: `git commit -m "Add your feature"`
7. **Push to your fork**: `git push origin feature/your-feature-name`
8. **Create a Pull Request**

## 📋 Development Guidelines

### Code Style
- Follow Rust formatting: `cargo fmt`
- Run clippy: `cargo clippy --all-targets --all-features`
- Write comprehensive tests for new functionality
- Document public APIs with rustdoc comments
- Follow the existing code patterns and architecture

### Commit Messages
Use conventional commits format:
- `feat(component): add new feature`
- `fix(component): fix bug description`
- `docs: update documentation`
- `test: add tests for feature`
- `refactor: improve code structure`

Examples:
- `feat(relayer): add event monitoring for new chain`
- `fix(api): handle websocket connection errors`
- `docs: update deployment guide`

### Testing Requirements
- All new features must include tests
- Maintain or improve test coverage
- Run the full test suite: `cargo test --workspace`
- Test both success and error scenarios
- Include integration tests for new components

### Documentation
- Update relevant documentation for new features
- Add rustdoc comments for public APIs
- Update README.md if needed
- Include examples in documentation

## 🔄 Pull Request Process

1. **Ensure all tests pass** locally
2. **Update documentation** if your changes affect the API or behavior
3. **Add tests** for new features or bug fixes
4. **Run code formatting**: `cargo fmt`
5. **Run linting**: `cargo clippy --all-targets --all-features`
6. **Create descriptive PR title** following conventional commits
7. **Fill out the PR template** completely
8. **Request review** from maintainers
9. **Address review feedback** promptly

### PR Checklist
- [ ] Tests pass locally (`cargo test --workspace`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings (`cargo clippy`)
- [ ] Documentation updated
- [ ] Tests added for new functionality
- [ ] PR description explains the changes
- [ ] Breaking changes are documented

## 🐛 Bug Reports

When reporting bugs, please include:
- **Clear description** of the issue
- **Steps to reproduce** the problem
- **Expected vs actual behavior**
- **Environment details** (OS, Rust version, etc.)
- **Relevant logs or error messages**
- **Minimal code example** if applicable

## 💡 Feature Requests

For new features:
- **Describe the use case** and motivation
- **Explain the proposed solution**
- **Consider backwards compatibility**
- **Discuss implementation approach**
- **Provide examples** of how it would be used

## 🔒 Security

If you discover a security vulnerability:
- **Do NOT create a public issue**
- **Email security concerns** to the maintainers
- **Provide detailed information** about the vulnerability
- **Allow time for assessment** and fix before disclosure

## 📚 Areas for Contribution

### High Priority
- Performance optimizations
- Additional blockchain support
- Enhanced security features
- Improved error handling
- Better monitoring and observability

### Documentation
- Tutorial improvements
- API documentation
- Deployment guides
- Troubleshooting guides
- Educational content

### Testing
- Integration test scenarios
- Performance benchmarks
- Security test cases
- Fuzzing implementations
- Mock improvements

### Infrastructure
- CI/CD improvements
- Docker optimizations
- Deployment automation
- Monitoring setup
- Development tooling

## 🎯 Code Review Guidelines

### For Contributors
- Keep PRs focused and reasonably sized
- Respond to feedback constructively
- Test your changes thoroughly
- Write clear commit messages
- Update documentation as needed

### For Reviewers
- Be constructive and helpful
- Focus on code quality and correctness
- Check for security implications
- Verify tests are adequate
- Ensure documentation is updated

## 🏗️ Development Setup

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-watch
cargo install cargo-audit
```

### Local Development
```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/cross-chain-bridge.git
cd cross-chain-bridge

# Run tests
cargo test --workspace

# Start development server
cargo run --bin api-server

# Watch for changes
cargo watch -x "test --workspace"
```

### Database Setup
```bash
# Install PostgreSQL
# Ubuntu/Debian: sudo apt install postgresql postgresql-contrib
# macOS: brew install postgresql

# Create development database
createdb bridge_dev

# Set environment variable
export DATABASE_URL=postgresql://localhost/bridge_dev
```

## 📞 Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Documentation**: Check the docs/ directory
- **Code Examples**: Look at the tests/ directory

## 🙏 Recognition

Contributors will be recognized in:
- CONTRIBUTORS.md file
- Release notes for significant contributions
- Project documentation
- GitHub contributor graphs

Thank you for contributing to the Cross-Chain Bridge project! 🌉
