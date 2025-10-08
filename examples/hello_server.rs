
use std::io::Read;
use clap::Parser;

use tracing_subscriber::filter::EnvFilter;

fn init_env_filter(env_filter: EnvFilter) {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_level(true)
        .with_target(true);

    let subscriber = subscriber.finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn main() -> std::io::Result<()> {
    // Initialize log bridge to capture log crate messages - MUST be first!
    tracing_log::LogTracer::init().expect("Failed to set logger");
    
    // Initialize tracing subscriber
    match EnvFilter::try_from_default_env() {
        Ok(env_filter) => init_env_filter(env_filter),
        _ => { }
     }
     
    let args = Args::parse();
    println!("Starting BondTcpListener server on {}", args.listen);    
    
    let mut listener = bond_tcp::BondTcpListener::bind(args.listen, args.bond)?;
    if let Ok((mut stream, addr)) =  listener.accept() {
        println!("Accepted connection from: {addr}");
        let mut buf = vec![0u8; args.size];

        loop {            
            let n = stream.read(&mut buf)?;
            if n == 0 {
                println!("Socket close from remote party...");
                break;
            }
            print!(".");
            // println!("Read {n} bytes:\n");
            // for b in buf.iter() {
            //     print!("{b}:");
            // }
            // println!("");

        }
    } else {
        println!("Failed to accept connection!");
    }
    Ok(())
}

/// A simple server illustrating the use of socket bonding.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The server listen address in the formant <ip:port>
    #[arg(short, long)]
    listen: String,
    /// The read buffer size
    #[arg(short, long)]
    size: usize,
    /// The number of socket streams to be bonded
    #[arg(short, long)]
    bond: u8
}