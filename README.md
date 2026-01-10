# rust-webapp

An educational Rust web application demonstrating a clean architecture approach with a `UserRepo` port and multiple storage adapters (SQLite, MongoDB, in-memory). Built with Actix Web, this project shows how to decouple business logic from persistence by defining a repository trait and swapping implementations.

## Quickstart

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)

### Run the Server

```bash
cargo run
```

The server starts on `http://127.0.0.1:8080`.

### API Documentation

OpenAPI/Swagger documentation is available at:

```
GET http://127.0.0.1:8080/v3/api-docs
```

### Configuration

The application uses a local SQLite database (`data.sqlite`) by default. No environment variables are required for basic usage.

## Project Structure

- **`src/main.rs`** – HTTP handlers (Actix Web routes), application startup, and OpenAPI schema
- **`src/app.rs`** – Application service layer; generic over `UserRepo`
- **`src/users.rs`** – `User` domain model and `UserRepo` trait (the port)
- **`src/adapters/`** – Repository implementations:
  - `sqlite.rs` – SQLite adapter using `sqlx`
  - `mongo.rs` – MongoDB adapter
  - `memory.rs` – In-memory adapter (HashMap-based)

## Adapters

The current `main.rs` is configured to use the **SQLite adapter**. To switch adapters:

1. Import the desired adapter (e.g., `MemoryUserRepo` or `MongoUserRepo`)
2. Instantiate it instead of `SqliteUserRepo` in `main()`
3. Update the `AppState` type accordingly

For example, to use the in-memory adapter:

```rust
use crate::adapters::memory::MemoryUserRepo;

let users_impl = MemoryUserRepo::new();
let application = Application::new(users_impl);
```

### Adding a New Adapter

1. Create a new file in `src/adapters/` (e.g., `postgres.rs`)
2. Implement the `UserRepo` trait for your struct
3. Export it from `src/adapters/mod.rs`
4. Wire it up in `main.rs`

## Tests

Run all tests:

```bash
cargo test
```

Tests run against all three adapters (memory, SQLite, MongoDB). The MongoDB tests use [testcontainers](https://crates.io/crates/testcontainers) to spin up a Docker container automatically.
