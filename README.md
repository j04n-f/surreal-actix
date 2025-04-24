# Rust Actix-web SurrealDB Onion Architecture Example

This project serves as a practical example of building a web application in Rust using Actix-web and SurrealDB, following the principles of **Onion Architecture**. It aims to demonstrate a clean, maintainable, testable, and decoupled application structure.

## Core Technologies

*   **Language:** [Rust](https://www.rust-lang.org/)
*   **Web Framework:** [Actix-web](https://actix.rs/)
*   **Database:** [SurrealDB](https://surrealdb.com/)
*   **Async Runtime:** [Tokio](https://tokio.rs/)
*   **Configuration:** `figment`
*   **Validation:** `validator`
*   **API Documentation:** `utoipa` (for OpenAPI)

## Architecture: Onion Architecture

The project adheres to the Onion Architecture principles to ensure a separation of concerns and a clear dependency flow. Dependencies always point inwards, towards the domain core.

*   **`src/domain`**: The core of the application. Contains business entities, value objects, and domain logic interfaces (traits). Has no dependencies on outer layers.
*   **`src/services`**: The application services layer. Orchestrates use cases by coordinating domain objects and infrastructure implementations (via traits). Depends only on `domain`.
*   **`src/infrastructure`**: Contains concrete implementations for external concerns like database access (SurrealDB repositories), external API clients, logging, etc. Implements traits defined in `domain` or `services`. Depends on `domain` and `services`.
*   **`src/api`**: Part of the infrastructure layer, specifically handling the Actix-web setup. Contains request handlers, routes, DTOs (Data Transfer Objects), request validation, and error handling specific to the web API. Depends on `services`.

For a detailed explanation of the layers and their responsibilities, please refer to the [Project Structure Rule](mdc:.cursor/rules/project-structure.mdc).

## Getting Started

### Prerequisites

*   **Rust Toolchain:** Install Rust via [rustup](https://rustup.rs/).
*   **SurrealDB:** A running SurrealDB instance. You can use Docker:

    ```bash
    docker run --rm --name surrealdb -p 8000:8000 surrealdb/surrealdb:latest start --log trace --user root --pass root memory
    ```
*   **watchexec-cli:** (A simple standalone tool that watches a path and runs a command whenever it detects modifications): `cargo install watchexec-cli`
*   **cargo-nextest:** (A next-generation test runner for Rust projects): `cargo install cargo-nextest`
*   **cargo-llvm-cov:** (Optional, for test coverage): `cargo install cargo-llvm-cov`

### Configuration

The application uses `figment` for configuration, loading settings in the following order:

1.  Defaults defined in `src/config.rs`.
2.  `config/default.toml`: Base configuration applicable to all environments.
3.  `config/{RUST_ENV}.toml`: Environment-specific overrides (e.g., `config/docker.toml`). The environment is determined by the `RUST_ENV` environment variable, which defaults to `development`.
4.  Environment variables: Prefixed with `APP_` and using `__` as a separator for nested values (e.g., `APP_SURREALDB__HOST`).

Refer to `src/config.rs` for details on configuration loading.

### Building

```bash
cargo build
```

### Running

```bash
cargo run
```
The server should start, typically on `http://127.0.0.1:8080`.

Run the Auto-Reloading Development Server:

```bash
watchexec -w src -r cargo run
```

### Testing

Run all tests:
```bash
cargo nextest run
```

Run tests with code coverage:
```bash
cargo llvm-cov --lcov nextest --output-path lcov.info
```

## Pre-commit Hooks

This project uses [pre-commit](https://pre-commit.com/) to automatically run checks (like formatting and linting) before each commit.

### Installation

1.  Install `pre-commit`:

    ```bash
    pip install pre-commit # or use your preferred package manager
    ```
2.  Install the git hooks:

    ```bash
    pre-commit install
    ```

Now, pre-commit will run automatically when you `git commit`.

You can also run all checks manually:

```bash
pre-commit run --all-files
```

The checks configured can be found in `.pre-commit-config.yaml`.

## Project Structure Overview

```
.
├── .cursor/rules/          # Cursor AI rules for development guidelines
├── .github/workflows/      # CI/CD workflows (e.g., rust.yml)
├── config/                 # Application configurations
├── migration/              # Database migration scripts
├── src/                    # Source code (following Onion Architecture)
│   ├── api/                # Actix-web layer (part of Infrastructure)
│   │   ├── controllers/    # Request handlers
│   │   ├── dto/            # Data Transfer Objects (request/response)
│   │   └── middlewares/    # Request/response middleware
│   ├── domain/             # Core domain layer (innermost layer)
│   │   ├── entities/       # Core business objects, value objects, aggregates
│   │   ├── models/         # Domain models
│   │   ├── repositories/   # Repository interfaces (traits)
│   │   ├── services/       # Domain service interfaces
│   │   └── error.rs        # Domain-specific error types
│   ├── infrastructure/     # Implementation of external concerns (outermost layer)
│   │   ├── databases/      # Database connection and management
│   │   ├── models/         # Infrastructure-specific models
│   │   └── repositories/   # Concrete database repository implementations
│   ├── services/           # Application services layer (use case orchestration)
│   ├── tests/              # Integration tests (using Testcontainers)
│   ├── app.rs              # Actix App configuration
│   ├── config.rs           # Configuration loading
│   ├── container.rs        # Dependency Injection container
│   ├── main.rs             # Application entry point
│   └── opentelemetry.rs    # Tracing/metrics setup
├── .dockerignore
├── .gitignore
├── .pre-commit-config.yaml # Pre-commit hooks configuration
├── .secrets.baseline       # Baseline for secret scanning
├── .surrealdb              # SurrealDB configuration
├── Cargo.lock
├── Cargo.toml              # Project dependencies
├── Dockerfile              # Container definition
├── README.md               # Project documentation
└── docker-compose.yaml     # Multi-container Docker setup
```

## API Documentation

API endpoints are documented using `utoipa`. The OpenAPI specification can typically be found at:

*   **Swagger UI:** `http://localhost:8080/swagger-ui/index.html`
