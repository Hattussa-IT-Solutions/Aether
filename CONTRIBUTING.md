# Contributing to Aether

Thank you for your interest in contributing to Aether!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/aether.git`
3. Create a branch: `git checkout -b feature/my-feature`
4. Make your changes
5. Run tests: `cargo test`
6. Commit: `git commit -m "Add my feature"`
7. Push: `git push origin feature/my-feature`
8. Open a Pull Request

## Development Setup

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run tests
cargo test

# Run an example
cargo run -- run examples/hello.ae

# Run clippy
cargo clippy
```

## Code Style

- Follow standard Rust conventions
- Use `cargo fmt` before committing
- All public functions need doc comments
- Write tests for new features

## Areas for Contribution

- **Language features** — New syntax, stdlib methods
- **Performance** — Bytecode VM optimizations, Cranelift codegen
- **Tooling** — VS Code extension, formatter, LSP
- **Documentation** — Examples, tutorials, API docs
- **Testing** — Edge cases, fuzzing, benchmarks
