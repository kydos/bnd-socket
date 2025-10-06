//! # BND Socket - TCP Connection Bonding Library
//!
//! BND Socket is a Rust library that provides transparent TCP connection bonding
//! to improve network throughput and resilience. By aggregating multiple TCP 
//! connections between the same endpoints, applications can achieve higher 
//! bandwidth utilization and better recovery from network congestion.
//!
//! ## Key Features
//!
//! - **Connection Bonding**: Automatically bonds multiple TCP connections from 
//!   the same source address into a single logical stream
//! - **Increased Throughput**: Utilizes multiple parallel connections to exceed 
//!   single-connection bandwidth limitations
//! - **Congestion Recovery**: Traffic continues flowing through unaffected 
//!   connections when individual connections experience issues
//! - **Path Diversity**: Multiple connections may take different network paths,
//!   providing resilience against path-specific problems
//! - **Transparent API**: Drop-in replacement for standard TCP listeners and 
//!   streams with familiar `std::net` interfaces
//!
//! ## How It Works
//!
//! The library works by accepting multiple TCP connections from the same client
//! and bonding them together once a configurable threshold is reached. Data 
//! written to a bonded stream is distributed across all underlying connections,
//! while data read from the stream is collected from all connections.
//!
//! ## Basic Usage
//!
//! ### Server Side
//!
//! ```rust,no_run
//! use bnd_socket::BondTcpListener;
//! use std::io::{Read, Write};
//!
//! // Create a listener that bonds 3 connections per client
//! let mut listener = BondTcpListener::bind("127.0.0.1:8080", 3)?;
//!
//! // Accept bonded connections
//! loop {
//!     let (mut stream, addr) = listener.accept()?;
//!     println!("Accepted bonded connection from {}", addr);
//!     
//!     // Use the bonded stream like a regular TCP stream
//!     let mut buffer = [0; 1024];
//!     let bytes_read = stream.read(&mut buffer)?;
//!     stream.write_all(&buffer[..bytes_read])?;
//! }
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! ### Client Side
//!
//! ```rust,no_run
//! use std::net::TcpStream;
//! use std::io::{Read, Write};
//! use std::thread;
//!
//! // Establish multiple connections to the same server
//! let handles: Vec<_> = (0..3).map(|_| {
//!     thread::spawn(|| -> std::io::Result<()> {
//!         let mut stream = TcpStream::connect("127.0.0.1:8080")?;
//!         stream.write_all(b"Hello from bonded connection!")?;
//!         
//!         let mut response = [0; 1024];
//!         let bytes_read = stream.read(&mut response)?;
//!         println!("Response: {}", String::from_utf8_lossy(&response[..bytes_read]));
//!         Ok(())
//!     })
//! }).collect();
//!
//! // Wait for all connections to complete
//! for handle in handles {
//!     handle.join().unwrap()?;
//! }
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! ## Configuration
//!
//! The number of connections to bond is configurable via the `stream_num` parameter
//! in `BondTcpListener::bind()`. Higher values provide more parallelism but require
//! clients to establish more connections.
//!
//! ## Use Cases
//!
//! - **High-throughput applications**: Where single TCP connection bandwidth 
//!   is insufficient
//! - **Unreliable networks**: Where connection redundancy improves reliability
//! - **Long-distance connections**: Where multiple paths can reduce latency 
//!   variance
//! - **Bulk data transfer**: Where parallel streams can saturate available 
//!   bandwidth
//!
//! ## Performance Considerations
//!
//! - Connection bonding works best when network paths have different 
//!   characteristics (latency, bandwidth, congestion)
//! - The optimal number of bonded connections depends on network conditions 
//!   and application requirements
//! - Overhead exists for managing multiple connections, so bonding may not 
//!   benefit very short-lived connections


#![warn(missing_docs)]

mod bond_tcp;
pub use bond_tcp::*;
