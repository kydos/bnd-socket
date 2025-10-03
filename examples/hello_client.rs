use tracing::{info, error};
use std::io::{Write, Read};
use std::thread;
use std::time::Duration;
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
     let env_filter = EnvFilter::try_from_default_env().unwrap();
    init_env_filter(env_filter);
    
    println!("Starting BondTcpStream client");
    println!("Creating 3 connections to 127.0.0.1:7890 to form a bond");
    let mut stream = bond_tcp::BondTcpStream::connect("127.0.0.1:7890")?;
    println!("Connected successfully");

    thread::sleep(Duration::from_secs(5));
    Ok(())
}