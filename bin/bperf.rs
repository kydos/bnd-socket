use clap::Parser;
use std::{io::Read, io::Write, time::{Duration, Instant}};
use tracing_subscriber::filter::EnvFilter;

fn run_client_mode(args: Args) { 

    let mut stream = bond_tcp::BondTcpStream::connect(args.addr.clone()).unwrap();
    println!("Connected successfully to {}", args.addr);
    
    let mut buf = vec![0u8; args.size];
    buf.fill(42);    

    loop {
        let n = stream.write(&buf).unwrap();
        if n == 0 { 
            println!("Connectio closed by remote peer");
            break;
        }        
    }    
}

fn run_server_mode(args: Args) { 
    
    let mut listener = bond_tcp::BondTcpListener::bind(args.addr, args.bond).unwrap();
    let mut sid = 0;
    loop {
        if let Ok((mut stream, addr)) =  listener.accept() {
            println!("Accepted connection from: {addr}");
            let mut buf = vec![0u8; args.size];
            let mut start = Instant::now();
            let mut total_recv = 0;
            let sampling_period = Duration::from_secs(args.period);
            let cid = sid;
            sid += 1;
            std::thread::spawn(move || {
            loop {            
                let n = stream.read(&mut buf).unwrap();

                if n == 0 {
                    println!("Socket close from remote party...");
                    break; 
                }          
                total_recv += n;
                let delta = start.elapsed();
                if  delta >= sampling_period {
                    let throughput = ((total_recv * 8) as f32 / delta.as_secs_f32()) / ( 10u64.pow(6) as f32);                    
                    println!("[{cid}]: {throughput} Mbps");
                    start = Instant::now();
                    total_recv = 0;
                }
            }});
        } else {
            println!("Failed to accept connection!");
        }
    }   
}

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

fn main() { 

     tracing_log::LogTracer::init().expect("Failed to set logger");
    
    // Initialize tracing subscriber
     match EnvFilter::try_from_default_env() {
        Ok(env_filter) => init_env_filter(env_filter),
        _ => { }
     }

    let args = Args::parse();
    if args.client {
        run_client_mode(args);
    } else {
        run_server_mode(args);
    }
}

/// The performance benchmarking application for BondSocket
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Set the client mode for the application
    #[arg(short, long)]
    client: bool,
    /// The address <ip:port> to listen or connect, depending on the mode.
    #[arg(short, long)]
    addr: String,
    /// The read buffer size
    #[arg(short, long)]
    size: usize,
    /// The number of socket streams to be bonded
    #[arg(short, long, default_value = "4")]
    bond: u8,
    /// The sampling period
    #[arg(short, long, default_value = "1")]
    period: u64,
}