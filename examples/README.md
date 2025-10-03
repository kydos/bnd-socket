# Bond TCP Examples

This directory contains example programs demonstrating the bond-tcp library functionality.

## Running Examples

### Method 1: Using the main project (with features)

From the project root:

```bash
# Build examples with tracing support
cargo build --examples --features examples

# Run the server (in one terminal)
RUST_LOG=debug cargo run --example hello_server --features examples

# Run the client (in another terminal)  
RUST_LOG=debug cargo run --example hello_client --features examples
```

### Method 2: Using the examples workspace

From the examples directory:

```bash
cd examples

# Build all examples
cargo build

# Run the server (in one terminal)
RUST_LOG=debug cargo run --bin hello_server

# Run the client (in another terminal)
RUST_LOG=debug cargo run --bin hello_client
```

## Logging

The examples use the `tracing` crate for structured logging. Set the `RUST_LOG` environment variable to control log levels:

- `RUST_LOG=error` - Only errors
- `RUST_LOG=warn` - Warnings and errors  
- `RUST_LOG=info` - Info, warnings, and errors
- `RUST_LOG=debug` - Debug, info, warnings, and errors
- `RUST_LOG=trace` - All log levels

## Examples

### hello_server.rs

A simple echo server that:
1. Binds to `127.0.0.1:7890` 
2. Waits for 3 TCP connections to bond together
3. Echoes back any received messages
4. Logs connection events and data flow

### hello_client.rs

A client that:
1. Creates 3 separate threads
2. Each thread connects to the server
3. All connections get bonded together on the server side
4. Sends test messages and receives echo responses
5. Logs the entire process

## Note

Currently, the `BondTcpStream::connect()` method contains `todo!()` placeholders. The examples demonstrate the intended API design but will panic when run until the implementation is completed.