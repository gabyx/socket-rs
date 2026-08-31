use anyhow::{Result, anyhow};
use clap::Parser;
use common::socket;

/// A simple CLI that says hello.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet.
    #[clap(short, long, default_value = socket::Side::A)]
    side: socket::Side,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Starting socket side '{:?}'.", args.side);

    Ok(())
}
