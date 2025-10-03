# BND Socket

A Rust library that provides bonded TCP streams with automatic load balancing, framing, and sequence numbering.

## Features

- **Round-robin distribution**: Data is sent across multiple TCP streams in a round-robin fashion
- **Automatic framing**: Data is automatically framed with sequence numbers to maintain order
- **Handshake protocol**: New streams are added with a handshake to ensure both sides are synchronized
- **TcpStream-compatible interface**: Implements `AsyncRead` and `AsyncWrite` traits for seamless integration
- **Sequence numbering**: Maintains sequence numbers to ensure data ordering across multiple streams

## Usage

### Basic Example

Run the server in one terminal:
```bash
cargo run --example basic_usage -- server
```

Run the client in another terminal:
```bash
cargo run --example basic_usage -- client
```

Or use the convenience script:
```bash
# Terminal 1
./run_example.sh server

# Terminal 2
./run_example.sh client
```

### Code Example

```rust
use bnd_socket::bndsock::BndTcpStream;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to multiple ports (same destination, different paths)
    let stream1 = TcpStream::connect("127.0.0.1:8080").await?;
    let stream2 = TcpStream::connect("127.0.0.1:8081").await?;
    
    // Create a BndTcpStream with initial streams
    let mut bnd_stream = BndTcpStream::new(vec![stream1, stream2]);
    
    // Add another stream with handshake
    let stream3 = TcpStream::connect("127.0.0.1:8082").await?;
    if bnd_stream.add_stream(stream3).await? {
        println!("Successfully added third stream");
    }
    
    // Use it like a regular TcpStream
    let data = b"Hello, World!";
    bnd_stream.write_all(data).await?;
    
    // Read response
    let mut buffer = [0u8; 1024];
    let bytes_read = bnd_stream.read(&mut buffer).await?;
    println!("Received: {:?}", &buffer[..bytes_read]);
    
    Ok(())
}
```

## How it works

1. **Length-Prefixed Protocol**: All messages (frames and handshakes) are sent with a 4-byte big-endian length prefix, ensuring exact message boundaries and preventing data corruption

2. **Framing**: Each write operation is wrapped in a `Frame` struct containing:
   - `sn`: Sequence number for ordering
   - `payload`: The actual data

3. **Round-robin distribution**: Data frames are distributed across available streams using a round-robin algorithm

4. **Handshake protocol**: When adding a new stream:
   - Sends an `ActivateLink` message with current sequence number (length-prefixed)
   - Waits for an `ActivateAck` response (length-prefixed)
   - Only adds the stream if handshake succeeds

5. **Reading**: Frames are read from any available stream:
   - First reads 4-byte length prefix
   - Then reads exactly that many bytes for the frame data
   - Deserializes and extracts the payload

## Protocol Messages

All messages use a length-prefixed format:
```
[4-byte length (big-endian)][message data]
```

- `Frame`: Contains sequence number and payload data
- `ActivateLink`: Handshake initiation message  
- `ActivateAck`: Handshake acknowledgment message

All messages are serialized using bincode for efficient binary encoding, with the serialized data preceded by its length.

## Running the Examples

The repository includes several examples:

1. **basic_usage.rs**: A complete server/client example
   ```bash
   # Terminal 1 (Server)
   cargo run --example basic_usage -- server
   
   # Terminal 2 (Client)
   cargo run --example basic_usage -- client
   ```

2. **Using the convenience script**:
   ```bash
   chmod +x run_example.sh
   ./run_example.sh server  # or ./run_example.sh client
   ```

The server will listen on ports 8080, 8081, and 8082, accepting connections and echoing back any received messages. The client will connect to all three ports, create a BndTcpStream, and send test messages.
Socket Bonding to improve throughput and ensure faster congestion control recovery.
