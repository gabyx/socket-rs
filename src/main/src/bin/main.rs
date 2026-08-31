use anyhow::{Context, Result};
use clap::Parser;
use common::comm;
use std::net;

/// A simple CLI that says hello.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet.
    #[clap(short, long, default_value = "a")]
    side: comm::Side,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Args: {args:?}");
    println!("Starting socket side '{:?}'.", args.side);

    let socket = start_socket(args.side)?;

    Ok(())
}

fn start_socket(side: comm::Side) -> Result<net::UdpSocket> {
    let address = format!("127.0.0.1:{}", side.port());
    println!("Binding UDP socket to '{address}'.");

    net::UdpSocket::bind(address.clone())
        .with_context(|| format!("Cannot bind socket on {address}"))
}
