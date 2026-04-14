# Clipstack

A minimal, local-first clipboard daemon for KDE Plasma and other clients. Built with Rust and designed as a modular workspace where each crate is a learnable, independent piece.

## Architecture

This workspace follows the "stairs not a ladder" philosophy — each crate is independently usable and comprehensible:

- **clipstack-core**: Data model and traits (no I/O)
- **clipstack-db**: SQLite implementation of the Store trait
- **clipstack-net**: HTTP API with axum
- **clipstack-daemon**: Binary that ties everything together

## Quick Start

### Prerequisites

- Rust 1.70 or higher
- SQLite (usually pre-installed on Linux)

### Build

```bash
cd clipstack
cargo build --release
```

### Run the daemon

```bash
# Default: binds to 127.0.0.1:8080, uses /tmp/clipstack.db
cargo run --release

# Custom database path
CLIPSTACK_DB_PATH=/path/to/db.sqlite cargo run --release

# Bind to all interfaces (for Tailscale)
CLIPSTACK_BIND_ADDRESS=0.0.0.0:8080 cargo run --release
```

## API Endpoints

### POST /push

Push clipboard content to the server:

```bash
curl -X POST http://localhost:8080/push \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Hello, world!",
    "mime_type": "text/plain",
    "device_id": "my-laptop"
  }'
```

### GET /latest

Get the latest clipboard entry:

```bash
curl http://localhost:8080/latest
```

### GET /history

Get clipboard history (all devices):

```bash
curl http://localhost:8080/history
```

### GET /history/:device_id

Get history for a specific device:

```bash
curl http://localhost:8080/history/my-laptop
```

### DELETE /entry/:id

Delete a specific entry:

```bash
curl -X DELETE http://localhost:8080/entry/<uuid>
```

## Environment Variables

- `CLIPSTACK_DB_PATH`: Path to SQLite database (default: `/tmp/clipstack.db`)
- `CLIPSTACK_BIND_ADDRESS`: Address to bind to (default: `127.0.0.1:8080`)
- `RUST_LOG`: Logging level for the daemon (default: `clipstack=info,tokio=info`)

## Development

### Running tests

```bash
cargo test
```

### Code formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Project Structure

```
clipstack/
├── clipstack-core/        # Data model and traits
│   └── src/lib.rs
├── clipstack-db/          # SQLite storage implementation
│   └── src/lib.rs
├── clipstack-net/         # HTTP API
│   └── src/lib.rs
├── clipstack-daemon/      # Binary
│   └── src/main.rs
├── Cargo.toml             # Workspace manifest
└── README.md
```

## Design Principles

1. **Modularity**: Each crate solves one well-defined problem
2. **Simplicity**: No unnecessary abstractions or frameworks
3. **Learnability**: Easy to understand and extend, even for beginners
4. **Local-first**: Everything runs locally, no cloud dependencies
5. **Stairs, not ladders**: Each layer builds on the previous without hiding complexity

## Future Enhancements

- WebSocket support for real-time sync
- Deduplication via content hashing
- Tagging and search capabilities
- Multiple device support with conflict resolution
- Integration with KDE Plasma Connect
- Android client

## License

MIT