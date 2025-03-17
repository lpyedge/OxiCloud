# OxiCloud Development Guide

## Build Commands
```bash
cargo build                 # Build the project
cargo run                   # Run the project locally (server at http://127.0.0.1:8085)
cargo test                  # Run all tests
cargo test -- --nocapture   # Run tests with output displayed
cargo test <test_name>      # Run a specific test (e.g., cargo test file_service)
cargo clippy                # Run linter to catch common mistakes
cargo clippy --fix          # Fix auto-fixable linting issues
cargo fmt --check           # Check code formatting without changing files
cargo fmt                   # Format code according to Rust conventions
RUST_LOG=debug cargo run    # Run with detailed logging for debugging
```

## Code Style Guidelines
- **Architecture**: Follow Clean Architecture layers (domain, application, infrastructure, interfaces)
- **Naming**: Use `snake_case` for files, modules, functions, variables; `PascalCase` for types/structs/enums
- **Modules**: Use mod.rs files for explicit exports with visibility modifiers (pub, pub(crate))
- **Error Handling**: Use Result<T, E> with thiserror for custom error types; propagate errors with ? operator
- **Comments**: Document public APIs with /// doc comments, explain "why" not "what"
- **Imports**: Group imports: 1) std, 2) external crates, 3) internal modules (with blank lines between)
- **Async**: Use async-trait for repository interfaces; handle futures with .await and tokio runtime
- **Testing**: Write unit tests in the same file as implementation (bottom of file, in a tests module)
- **Dependencies**: Use axum for web API, tower-http for middleware, serde for serialization
- **Logging**: Use tracing crate with appropriate levels (debug, info, warn, error)
- **Repository Pattern**: Define interfaces in domain layer, implement in infrastructure layer
- **I18n**: Store translations in JSON files under static/locales/, use i18n service for text lookups

## Project Structure
OxiCloud is a NextCloud-like file storage system built in Rust with a focus on performance and security. It provides a clean REST API and web interface for file management using a layered architecture approach. The roadmap in TODO-LIST.md outlines planned features including enhanced folder support, file previews, user authentication, sharing, and a sync client.