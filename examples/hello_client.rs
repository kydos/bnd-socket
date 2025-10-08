use std::time::Duration;
use std::io::Write;
use tracing_subscriber::filter::EnvFilter;
use clap::Parser;

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
    println!("Starting BondTcpStream client");
    let mut stream = bond_tcp::BondTcpStream::connect(args.connect)?;
    println!("Connected successfully");
    
    let mut buf = vec![0u8; args.size];

    loop {
        let n = stream.write(&buf)?;
        println!("Wrote {n} bytes");
        for i in 0..buf.len() {
            buf[i] = ((buf[i] + 1) %255) as u8
        }
        std::thread::sleep(Duration::from_micros(args.period));
    }    
}

/// A simple client illustrating the use of socket bonding.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The server address to connect to in the formant <ip:port>
    #[arg(short, long)]
    connect: String,
    /// The write buffer size
    #[arg(short, long)]
    size: usize,
    /// Production period in micro-sec
    #[arg(short, long)]
    period: u64
}