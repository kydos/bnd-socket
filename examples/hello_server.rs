
use tracing::{debug, info, warn, error};
use tracing_subscriber::filter::EnvFilter;
use std::io::{Read, Write};

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
    
    println!("Starting BondTcpListener server on 127.0.0.1:7890");
    println!("Waiting for 3 connections to bond together");
    
    let mut listener = bond_tcp::BondTcpListener::bind("127.0.0.1:7890", 3)?;
    if let Ok((_, addr)) =  listener.accept() {
        println!("Accepted connection from: {addr}");
    } else {
        println!("Failed to accept connection!");
    }
    Ok(())
}