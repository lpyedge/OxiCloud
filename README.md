# 🚀 OxiCloud

![OxiCloud](static/oxicloud-logo.svg)

## The high-performance, Rust-powered file storage solution

OxiCloud is a NextCloud-like file storage system built with Rust, designed from the ground up with **performance**, **security**, and **scalability** as its core principles. Perfect for self-hosting your own cloud storage or deploying in enterprise environments.

## ✨ Key Features

- 🔥 **Blazing Fast Performance**: Built with Rust and optimized for speed
- 📁 **Advanced File Management**: Intuitive folder structure with powerful batch operations
- 🔄 **Concurrent Processing**: Parallel file operations for large files and batch processing
- 🔍 **Smart Caching**: Multi-layered caching system for metadata and file access
- 🌐 **Internationalization**: Full i18n support (currently English and Spanish)
- 📱 **Responsive Design**: Works seamlessly on desktop and mobile devices
- 🔌 **Extensible Architecture**: Clean, layered design following domain-driven principles

## 🚀 Performance Optimizations

OxiCloud incorporates multiple advanced performance optimizations:

### Concurrency and Parallelism
- **Parallel File Processing**: Automatically splits large files into chunks for parallel processing
- **Asynchronous I/O**: Built on Tokio for non-blocking operations
- **Worker Pools**: Smart thread management for optimal resource utilization
- **Timeout Management**: Strategic timeouts to prevent resource exhaustion

### Intelligent Caching
- **File Metadata Cache**: Drastically reduces filesystem calls
- **Smart Cache Invalidation**: Selectively invalidates cache entries
- **Preloading**: Strategic preloading for frequently accessed directories
- **TTL-Based Cache**: Time-based expiration for optimal memory usage

### I/O Optimization
- **Buffer Pooling**: Reuses memory buffers to reduce allocation pressure
- **Adaptive Streaming**: Adjusts chunk sizes based on file size
- **Size-Based Processing**: Different strategies for small, medium, and large files
- **Non-Blocking Filesystem Operations**: Prevents I/O bottlenecks

### Batch Processing
- **ID Mapping Optimizer**: Groups mapping operations to reduce overhead
- **Operation Batching**: Processes multiple file operations concurrently
- **Debounced Saving**: Groups write operations for optimal I/O
- **Parallel Directory Scanning**: Efficient directory traversal

## 🧠 Advanced Technical Features

### Clean Architecture Implementation
- **Hexagonal/Ports and Adapters Pattern**: Clear separation between domain, application, and infrastructure
- **Dependency Inversion**: Domain business rules are independent of external frameworks
- **Explicit Dependency Injection**: Manual, type-safe DI without heavy frameworks

### Advanced Error Handling
- **Domain-Specific Error Types**: Granular error classification with context
- **Error Propagation Chain**: Preserves context through abstraction layers
- **Custom Error Context**: Enriches errors with additional information
- **Source Tracking**: Errors maintain their original source for debugging

### Robust Repository Pattern
- **Persistence Abstraction**: Domain layer completely isolated from storage details
- **Repository Interfaces**: Defined in domain layer and implemented in infrastructure
- **Storage Mediator Pattern**: Coordinates interactions between repositories
- **ID Mapping Service**: Decouples domain identifiers from filesystem paths

### Transaction Management
- **Atomic Operations**: Entity-level transaction support
- **Pending Changes System**: Batches persistence operations for efficiency
- **Rollback Capabilities**: Reverts state on failed operations
- **Optimistic Concurrency**: Protects against concurrent modifications

### Advanced File System Handling
- **Parallel Processing for Large Files**: Chunked operations for efficient I/O
- **Specialized Strategies**: Different handlers for various file sizes
- **Timeout-Protected Operations**: Prevents hanging on problematic files
- **Background Processing**: Heavy operations offloaded to background tasks

### Memory Efficiency
- **Buffer Pool Manager**: Reuses allocated memory to reduce fragmentation
- **Streaming I/O**: Processing large files without loading entirely into memory
- **Resource-Aware Processing**: Adapts resource usage based on file size
- **Lazy Loading**: Loads data only when needed

### Defensive Programming
- **Extensive Input Validation**: Domain entities enforce business rules
- **Immutable Data Structures**: Prevents unexpected state mutations
- **Fail-Fast Operations**: Early validation to prevent cascading failures
- **Extensive Logging**: Structured logs with contextual information

### Service Layer Optimizations
- **Application Services**: Orchestrate use cases with domain entities
- **Transaction Coordination**: Ensures data consistency across operations
- **Domain Service Specialization**: Services focused on specific domain concerns
- **Cross-Cutting Concerns**: Separated into dedicated middleware components

## 📸 Screenshots

*Coming soon!*

## 🛠️ Getting Started

### Prerequisites
- Rust 1.70+ and Cargo

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/oxicloud.git
cd oxicloud

# Build the project
cargo build --release

# Run the server
cargo run --release
```

The server will be available at `http://localhost:8085`

## 🧩 Project Structure

OxiCloud follows Clean Architecture principles with clear separation of concerns:

- **Domain Layer**: Core business logic and entities
- **Application Layer**: Use cases and application services
- **Infrastructure Layer**: External systems and implementations
- **Interfaces Layer**: API and web controllers

## 🚧 Development

```bash
# Core development workflow
cargo build                 # Build the project
cargo run                   # Run the project locally
cargo check                 # Quick check for compilation errors

# Testing
cargo test                  # Run all tests
cargo test <test_name>      # Run a specific test

# Code quality
cargo clippy                # Run linter
cargo fmt                   # Format code

# Debugging
RUST_LOG=debug cargo run    # Run with detailed logging
```

## 🗺️ Roadmap

OxiCloud is under active development. Upcoming features include:

- User authentication and multi-user support
- File sharing and collaboration features
- WebDAV support and sync clients
- File versioning
- Encryption
- Mobile applications

See [TODO-LIST.md](TODO-LIST.md) for a detailed roadmap.

## 🤝 Contributing

Contributions are welcome! Whether it's bug reports, feature suggestions, or code contributions, please feel free to reach out.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📜 License

OxiCloud is available under the MIT License. See the LICENSE file for more information.

## 🙏 Acknowledgements

- The Rust community for the amazing ecosystem
- All contributors who have helped shape this project

---

Designed with ❤️ by OxiCloud Team