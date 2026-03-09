# Contributing to Mini Gateway

Thank you for your interest in contributing to Mini Gateway! This document provides guidelines and information to help you contribute effectively.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [AI-Assisted Code Policy](#ai-assisted-code-policy)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Code Style](#code-style)
- [Pull Request Process](#pull-request-process)
- [Issue Guidelines](#issue-guidelines)
- [License](#license)

## Code of Conduct

This project adheres to the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code. Please read it before contributing.

## How Can I Contribute?

There are several ways to contribute to Mini Gateway:

- **Report Bugs** -- Found something broken? Open a [Bug Report](https://github.com/zonblade/mini-gateway-rs/issues/new?template=bug_report.yml).
- **Suggest Features** -- Have an idea? Submit a [Feature Request](https://github.com/zonblade/mini-gateway-rs/issues/new?template=feature_request.yml).
- **Propose Milestone Work** -- Want to tackle a larger piece of the roadmap? Open a [Milestone Implementation](https://github.com/zonblade/mini-gateway-rs/issues/new?template=milestone_implementation.yml) proposal.
- **Request Removal** -- Think something should be removed or deprecated? File a [Removal Request](https://github.com/zonblade/mini-gateway-rs/issues/new?template=request_removal.yml).
- **Report Security Vulnerabilities** -- Please see our [Security Policy](SECURITY.md). Do **not** open a public issue for security vulnerabilities.
- **Improve Documentation** -- Typos, unclear explanations, missing docs -- all welcome.
- **Submit Code** -- Bug fixes, performance improvements, new features.

## AI-Assisted Code Policy

We have a strict policy regarding AI-generated code:

- **Minimum 90% human-written code is required.** Contributions must be predominantly written by a human. AI tools (such as Copilot, ChatGPT, Claude, or similar) may be used for assistance, but the contributor must understand, review, and take full responsibility for every line of code submitted.
- **AI assistance must be disclosed.** If you used AI tools in any part of your contribution, you must clearly state this in your pull request description. Include:
  - Which AI tool(s) you used
  - What parts of the code were AI-assisted
- **You are responsible for your code.** Using AI does not exempt you from understanding what your code does. If you cannot explain your contribution line by line during review, it may be rejected.
- **Why this policy?** We value thoughtful, well-understood contributions. AI tools can be helpful, but over-reliance on them can lead to subtle bugs, security issues, and code that no one truly understands. Contributors who understand their code produce better software.

Pull requests that appear to be predominantly AI-generated without disclosure will be rejected.

## Development Setup

### Prerequisites

- Rust (latest stable) -- install via [rustup](https://rustup.rs/)
- Cargo (comes with Rust)
- Git

### Building

```bash
# Clone the repository
git clone https://github.com/zonblade/mini-gateway-rs.git
cd mini-gateway-rs

# Build all workspace members
cargo build

# Build in release mode
cargo build --release
```

### Running

Each workspace member runs independently. Refer to the README in each sub-directory for specific instructions:

- [`router-core/README.md`](router-core/README.md) -- Core proxy service
- [`router-api/README.md`](router-api/README.md) -- API interface
- `router-cli/` -- Command-line interface
- `router-rds/` -- RDS module

### Testing

```bash
# Run all tests
cargo test

# Run tests for a specific workspace member
cargo test -p router-core
```

## Project Structure

```
mini-gateway-rs/
├── router-core/     # Core proxy service (traffic routing and forwarding)
├── router-api/      # API interface for management and configuration
├── router-cli/      # Command-line interface
├── router-rds/      # RDS module
├── build-apt/       # APT package build files
├── build-docker/    # Docker build files
├── test-app/        # Test applications
├── test-docker/     # Docker test configurations
└── assets/          # Project assets (logo, diagrams)
```

## Code Style

- Follow standard Rust conventions and idioms.
- Run `cargo fmt` before committing -- all code must be formatted with `rustfmt`.
- Run `cargo clippy` and address any warnings before submitting.
- Write meaningful commit messages. We follow [Conventional Commits](https://www.conventionalcommits.org/) format:
  - `feat(scope): add new feature`
  - `fix(scope): fix specific bug`
  - `docs(scope): update documentation`
  - `refactor(scope): refactor code without changing behavior`
  - `test(scope): add or update tests`
  - `chore(scope): maintenance tasks`

## Pull Request Process

1. **Fork the repository** and create your branch from `main`.
2. **One PR per concern.** Keep pull requests focused on a single change. Do not bundle unrelated changes.
3. **Write a clear description.** Explain what your PR does and why. Reference any related issues.
4. **Disclose AI usage.** If you used AI tools, state it clearly in the PR description (see [AI-Assisted Code Policy](#ai-assisted-code-policy)).
5. **Ensure your code compiles** with `cargo build` and passes `cargo clippy` with no warnings.
6. **Format your code** with `cargo fmt`.
7. **Add tests** for new functionality where applicable.
8. **Keep it reviewable.** If your change is large, consider breaking it into smaller PRs.
9. **Be responsive.** Address review feedback in a timely manner.

### PR Review Criteria

Your pull request will be evaluated on:

- Correctness and functionality
- Code quality and readability
- Adherence to project conventions
- Test coverage (where applicable)
- Documentation for new features
- AI disclosure compliance

## Issue Guidelines

We provide issue templates to help you provide the right information:

- **Bug Report** -- For reporting bugs. Include reproduction steps, expected vs actual behavior, and environment details.
- **Feature Request** -- For suggesting new features. Explain the use case and proposed solution.
- **Milestone Implementation** -- For proposing work on roadmap items. Include your implementation plan.
- **Removal Request** -- For requesting removal or deprecation of features/components. Explain the rationale.
- **Security Issues** -- Do **not** use public issues. Follow our [Security Policy](SECURITY.md) instead.

## License

By contributing to Mini Gateway, you agree that your contributions will be licensed under the [Apache License 2.0](LICENSE), the same license that covers the project.

---

Questions? Feel free to open a discussion or reach out via an issue. We appreciate your contribution!
