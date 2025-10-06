use std::collections::HashMap;
use std::io::{Read, Result as IoResult, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, ToSocketAddrs};
use std::time::Duration;

use uuid::Uuid;

/// A TCP listener that bonds multiple connections from the same source address.
///
/// `BondTcpListener` provides a transparent way to aggregate multiple TCP/IP connections
/// to improve throughput and recovery from network congestion. When a client establishes
/// multiple connections to the same server address, this listener will automatically
/// bond them together into a single logical `BndTcpStream`.
///
/// # Benefits
///
/// - **Increased Throughput**: By utilizing multiple TCP connections in parallel,
///   the effective bandwidth can exceed what a single connection might achieve
/// - **Congestion Recovery**: If one connection experiences congestion or packet loss,
///   traffic can continue flowing through other connections in the bond
/// - **Path Diversity**: Multiple connections may take different network paths,
///   providing resilience against path-specific issues
/// - **Transparent Operation**: Applications can use bonded connections as if they
///   were regular TCP streams
///
/// # How It Works
///
/// The listener accepts incoming connections and groups them by source address.
/// When the configured number of streams (`stream_num`) from the same source
/// address is reached, they are bonded together and returned as a single
/// `BndTcpStream`. Until then, partial connections are held in an internal
/// buffer waiting for additional connections from the same source.
///
/// # Example
///
/// ```rust,no_run
/// use std::net::SocketAddr;
/// use bond_tcp::BondTcpListener;
///
/// // Create a listener that bonds 3 connections per source address
/// let mut listener = BondTcpListener::bind("127.0.0.1:8080", 3)?;
///
/// // Accept bonded connections
/// loop {
///     let (bonded_stream, addr) = listener.accept()?;
///     println!("Accepted bonded connection from {}", addr);
///     
///     // The bonded_stream now represents 3 TCP connections
///     // working together transparently
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # Configuration
///
/// The `stream_num` parameter determines how many individual TCP connections
/// must be established from the same source address before they are bonded
/// together. Higher values provide more parallelism but require the client
/// to establish more connections.
///
/// # Note
///
/// Clients must establish exactly `stream_num` connections to the same
/// server address to complete the bonding process. The listener will
/// block on `accept()` until all required connections are established.
pub struct BondTcpListener {
    listener: TcpListener,
    stream_num: u8,
    accepted_connections: HashMap<uuid::Uuid, std::vec::Vec<TcpStream>>,
}

const FRAGMENT_SIZE: usize = 8192;

impl BondTcpListener {
    /// Creates a new `BndTcpListener` which will be bound to the specified address.
    pub fn bind<A: ToSocketAddrs>(addr: A, stream_num: u8) -> IoResult<BondTcpListener> {
        let listener = TcpListener::bind(addr)?;
        Ok(BondTcpListener {
            listener,
            stream_num,
            accepted_connections: HashMap::new(),          
        })
    }    /// Returns the local address that this listener is bound to.
    pub fn local_addr(&self) -> IoResult<SocketAddr> {
        self.listener.local_addr()        
    }

    /// Creates a new independently owned handle to the underlying socket.
    pub fn try_clone(&self) -> IoResult<BondTcpListener> {
        Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "The BondTcpListener cannot be cloned"))        
    }

    /// Accept a new incoming connection from this listener.
    pub fn accept(&mut self) -> IoResult<(BondTcpStream, SocketAddr)> {
        let r_poller = polling::Poller::new().unwrap();
        let w_poller = polling::Poller::new().unwrap();
        loop {
            let mut cid_buf = [0u8; 16];
            let (mut stream, addr) = self.listener.accept()?;
            log::debug!("Accepted connection from: {addr}");
            let n = stream.read(&mut cid_buf)?;
            log::debug!("Read {n} bytes for CID");
            let cid = uuid::Uuid::from_bytes_le(cid_buf);
            log::debug!("Connection Id: {cid}");
            match self.accepted_connections.remove(&cid) {
                Some(mut streams) => {                     
                    if streams.len() + 1 == self.stream_num as usize {
                        log::debug!("We have already {} connections with {cid} accepting the session", streams.len());
                        streams.push(stream);                        
                        let mut id = 0;
                        for s in streams.iter() {
                            s.set_nonblocking(true)?;
                            unsafe {
                                let _ = r_poller.add(s, polling::Event::none(id));
                                let _ = w_poller.add(s, polling::Event::none(id));
                            }                                                  
                            id += 1;
                        }
                        return Ok((BondTcpStream { streams, r_poller, w_poller, next_stream: 0, readable: 0 }, addr));
                    }
                    else {
                        log::debug!("{} connection with {cid}", streams.len() + 1);    
                        // stream.set_nonblocking(true)?;
                        // unsafe {
                        //     let _ = r_poller.add(&stream, polling::Event::none(streams.len()));
                        //     let _ = w_poller.add(&stream, polling::Event::none(streams.len()));
                        // }                                                  
                        streams.push(stream);                
                        self.accepted_connections.insert(cid, streams);                         
                    }
                },
                None => {
                    // stream.set_nonblocking(true)?;
                    // unsafe {
                    //     let _ = r_poller.add(&stream, polling::Event::none(0));
                    //     let _ = w_poller.add(&stream, polling::Event::none(0));
                    // }

                    let cid = uuid::Uuid::new_v4();
                    log::debug!("First connection with {addr} associating it with cid: {cid}");                    
                    // Inform the other side about the number of socket to be opened.
                    let ns = self.stream_num.to_le_bytes();                        
                    log::debug!("Sending # of streams {}", self.stream_num);                    
                    stream.write_all(&ns)?;                                        
                    let cid_buf = cid.to_bytes_le();
                    stream.write_all(&cid_buf)?;
                    stream.flush()?;
                    log::debug!("Sending Cid");                    
                    self.accepted_connections.insert(cid, vec![stream]);                        
                }

            }
        }
                        
    }

    /// Returns an iterator over the connections being received on this listener.
    pub fn incoming(&self) -> Incoming<'_> {
        // TODO: Implement incoming
        todo!()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    pub fn set_ttl(&self, _ttl: u32) -> IoResult<()> {
        // TODO: Implement set_ttl
        todo!()
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    pub fn ttl(&self) -> IoResult<u32> {
        // TODO: Implement ttl
        todo!()
    }

    /// Sets the value for the `SO_REUSEADDR` option on this socket.
    pub fn set_nonblocking(&self, _nonblocking: bool) -> IoResult<()> {
        // TODO: Implement set_nonblocking
        todo!()
    }

    /// Gets the value of the `SO_REUSEADDR` option on this socket.
    pub fn take_error(&self) -> IoResult<Option<std::io::Error>> {
        // TODO: Implement take_error
        todo!()
    }
}

/// An iterator that infinitely accepts connections on a `BndTcpListener`.
pub struct Incoming<'a> {
    _listener: &'a BondTcpListener,
}

impl<'a> Iterator for Incoming<'a> {
    type Item = IoResult<BondTcpStream>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: Implement iterator
        todo!()
    }
}

/// A bonded TCP stream that aggregates multiple underlying TCP connections.
///
/// This struct represents multiple TCP connections that have been bonded together
/// to act as a single logical stream. Data written to this stream is distributed
/// across the underlying connections, and data read from this stream is collected
/// from all connections.
///
/// `BondTcpStream` provides the same interface as a standard `TcpStream` but with
/// the performance benefits of multiple parallel connections.
pub struct BondTcpStream {
    streams: std::vec::Vec<TcpStream>,
    r_poller: polling::Poller,
    w_poller: polling::Poller,
    next_stream: usize,
    readable: usize
}

impl BondTcpStream {
    /// Opens a TCP connection to a remote host.
    
    pub fn connect<A: ToSocketAddrs>(addr: A) -> IoResult<BondTcpStream> {
        let r_poller = polling::Poller::new().unwrap();
        let w_poller = polling::Poller::new().unwrap();
        let mut addresses = vec![]; 
        for a in addr.to_socket_addrs().unwrap() {
            addresses.push(a.clone());
        }        
        let tid = uuid::Uuid::new_v4();
        let mut stream = TcpStream::connect(addresses.as_slice())?;        
        // stream.set_nonblocking(true)?;
        // unsafe {
        //     let _ = r_poller.add(&stream, polling::Event::none(0));
        //     let _ = w_poller.add(&stream, polling::Event::none(0));
        // }

        log::debug!("Established first connection, sending challenge");            
        stream.write(&tid.to_bytes_le())?;        
        let _ = stream.flush();
        let mut len_buf = [0u8; size_of::<u8>()];
        let _ = stream.read(&mut len_buf)?;                
        let ns = u8::from_le_bytes(len_buf);
        let mut cid_buf = [0u8; 16];
        let _ = stream.read_exact(&mut cid_buf)?;

        log::debug!("BondTcpStream will open {ns} streams");
        log::debug!("CID: {}", Uuid::from_bytes_le(cid_buf.clone()));
        let mut streams = vec![stream];
        
        for _ in 1..ns {           
            log::debug!("Establishing another connection");
            let mut s = TcpStream::connect(addresses.as_slice())?;            
            // s.set_nonblocking(true)?;
            // unsafe {                
            //     let _ = r_poller.add(&s, polling::Event::none(i as usize));
            //     let _ = w_poller.add(&s, polling::Event::none(i as usize));
            // }
            log::debug!("Sending UUID: {}", Uuid::from_bytes_le(cid_buf.clone()));
            let _ = s.write(&cid_buf)?;
            let _ = s.flush();
            streams.push(s);            
        }
        
        let mut id = 0;
        for s in streams.iter() {
            let _ = s.set_nonblocking(true);
            unsafe {
                let _ = r_poller.add(s, polling::Event::none(id));
                let _ = w_poller.add(s, polling::Event::none(id));
                id += 1;
            }
        }
        Ok (BondTcpStream { streams, r_poller, w_poller, next_stream: 0, readable: 0 })        
    }

    /// Opens a TCP connection to a remote host with a timeout.
    pub fn connect_timeout(_addr: &SocketAddr, _timeout: Duration) -> IoResult<BondTcpStream> {
        // TODO: Implement connect_timeout
        todo!()
    }

    /// Returns the socket address of the remote peer of this TCP connection.
    pub fn peer_addr(&self) -> IoResult<SocketAddr> {
        // TODO: Implement peer_addr
        todo!()
    }

    /// Returns the socket address of the local half of this TCP connection.
    pub fn local_addr(&self) -> IoResult<SocketAddr> {
        // TODO: Implement local_addr
        todo!()
    }

    /// Shuts down the read, write, or both halves of this connection.
    pub fn shutdown(&self, _how: std::net::Shutdown) -> IoResult<()> {
        // TODO: Implement shutdown
        todo!()
    }

    /// Creates a new independently owned handle to the underlying socket.
    pub fn try_clone(&self) -> IoResult<BondTcpStream> {
        // TODO: Implement try_clone
        todo!()
    }

    /// Sets the read timeout to the timeout specified.
    pub fn set_read_timeout(&self, _dur: Option<Duration>) -> IoResult<()> {
        // TODO: Implement set_read_timeout
        todo!()
    }

    /// Sets the write timeout to the timeout specified.
    pub fn set_write_timeout(&self, _dur: Option<Duration>) -> IoResult<()> {
        // TODO: Implement set_write_timeout
        todo!()
    }

    /// Returns the read timeout of this socket.
    pub fn read_timeout(&self) -> IoResult<Option<Duration>> {
        // TODO: Implement read_timeout
        todo!()
    }

    /// Returns the write timeout of this socket.
    pub fn write_timeout(&self) -> IoResult<Option<Duration>> {
        // TODO: Implement write_timeout
        todo!()
    }

    /// Receives data on the socket from the remote address to which it is connected.
    pub fn peek(&self, _buf: &mut [u8]) -> IoResult<usize> {
        // TODO: Implement peek
        todo!()
    }

    /// Sets the value of the `TCP_NODELAY` option on this socket.
    pub fn set_nodelay(&self, _nodelay: bool) -> IoResult<()> {
        // TODO: Implement set_nodelay
        todo!()
    }

    /// Gets the value of the `TCP_NODELAY` option on this socket.
    pub fn nodelay(&self) -> IoResult<bool> {
        // TODO: Implement nodelay
        todo!()
    }

    /// Sets the value for the `IP_TTL` option on this socket.
    pub fn set_ttl(&self, _ttl: u32) -> IoResult<()> {
        // TODO: Implement set_ttl
        todo!()
    }

    /// Gets the value of the `IP_TTL` option for this socket.
    pub fn ttl(&self) -> IoResult<u32> {
        // TODO: Implement ttl
        todo!()
    }

    /// Get the value of the `SO_ERROR` option on this socket.
    pub fn take_error(&self) -> IoResult<Option<std::io::Error>> {
        // TODO: Implement take_error
        todo!()
    }

    /// Moves this TCP stream into or out of nonblocking mode.
    pub fn set_nonblocking(&self, _nonblocking: bool) -> IoResult<()> {
        // TODO: Implement set_nonblocking
        todo!()
    }

    fn write_loop(&mut self, buf: &[u8]) -> IoResult<usize> {        
        let n = self.streams[self.next_stream].write(buf)?;        
        if n < buf.len() {
            let mut index = n;
            let mut events = polling::Events::new();
            loop {
                events.clear();
                let _ = self.w_poller.modify(&self.streams[self.next_stream], polling::Event::writable(self.next_stream));
                let _ = self.w_poller.wait(&mut events, None)?;
                for e in events.iter() {
                    if e.key == self.next_stream {
                        let n = self.streams[self.next_stream].write(&buf[index..buf.len()])?;
                        index += n;                    
                    }
                    if index == buf.len() {
                        break;
                    }
                }                                    
            }
        }            
        Ok(buf.len())
    }

    fn read_loop(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let mut n = match self.streams[self.next_stream].read(buf) {
            Ok(n) => n,
            Err(_) => 0
        };
        let len = buf.len();        
        let mut events = polling::Events::new();
        while n < len {
            events.clear();
            self.r_poller.modify(
                &self.streams[self.next_stream], 
                polling::Event::readable(self.next_stream))?;
            let _ = self.r_poller.wait(&mut events, None)?;
            for e in events.iter() {
                if e.key == self.next_stream {
                    n += self.streams[self.next_stream].read(&mut buf[n..len])?;                    
                }
            }
        }
        Ok(buf.len())
    }
    fn read_frame_len(&mut self) -> IoResult<usize> {
        let mut len_bs = [0u8; 4]; 
        let _ = self.read_loop(&mut len_bs)?;
        let len = u32::from_le_bytes(len_bs) as usize;
        Ok(len)
    }

    fn read_readable(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        if self.readable > 0  {        
            let len = std::cmp::min(self.readable, buf.len());    
            let n = self.read_loop(&mut buf[0..len])?;
            if n == self.readable {
                self.next_stream = (self.next_stream + 1) % self.streams.len();
                self.readable = 0;
            } else {
                self.readable -= n;
            }
            Ok(n)
        } else {
            Ok(0)
        }
    }

}

impl std::io::Read for BondTcpStream {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {        
        log::debug!("Reading using stream {}", self.next_stream);
        let buf_len = buf.len();
        let mut n = self.read_readable(buf)?;
        while n < buf.len() {
            let len = self.read_frame_len()?;
            log::debug!("Frame Len: {len}");
            if len > buf.len() - n {
                self.readable = len - (buf.len() - n);
                n += self.read_loop(&mut buf[n..buf_len])?;                
            } else {
                self.readable = 0;                
                n += self.read_loop(&mut buf[n..buf_len])?;
                self.next_stream = (self.next_stream + 1) % self.streams.len();                
            }
        }       
        self.next_stream = (self.next_stream + 1) % self.streams.len();                
        log::debug!("Read  {} bytes, next will read from stream {}/{}", buf.len(), self.next_stream, self.streams.len());
        
        Ok(buf.len())
    }
}

impl std::io::Write for BondTcpStream {


    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        log::debug!("Writing using stream {}", self.next_stream);
        if buf.len() < FRAGMENT_SIZE {
            let len_bs = (buf.len() as u32).to_le_bytes();                        
            let _ = self.write_loop(&len_bs)?;
            let _ = self.write_loop(buf)?;            
        } else {            
            let mut sup = FRAGMENT_SIZE;
            let mut k = 0;
            while sup < buf.len() {
                let inf = k * FRAGMENT_SIZE;
                k += 1;
                sup = std::cmp::min(k*FRAGMENT_SIZE, buf.len());
                let _ = self.write_loop(&buf[inf..sup])?;                
            }            
        }
        self.next_stream = (self.next_stream +1 ) % self.streams.len();
        Ok(buf.len())
         
    }

    fn flush(&mut self) -> IoResult<()> {
        // TODO: Implement flush
        todo!()
    }
}